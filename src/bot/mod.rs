mod callbacks;
pub mod enums;
mod filters;
mod handlers;
mod keyboards;
mod libs;
mod methods;
pub mod middlewares;
mod task;
mod utils;

use crate::bot::handlers::register_routers;
use crate::bot::task::verbal_warns_clear_task;
use crate::config::Config;
use axum::Router as AxumRouter;
use sea_orm::DatabaseConnection;
use std::fmt::Display;
use telers::enums::UpdateType;
use telers::event::simple;
use telers::methods::SetWebhook;
use telers::webhooks::axum::{get_updates_router, UpdatesHandler};
use telers::{Bot, Dispatcher, Router};
use tokio::net::TcpListener;
use tokio::sync::broadcast::{channel, Receiver, Sender};

async fn set_webhook(
    bot: Bot,
    webhook_url: impl Display,
    webhook_path: impl Display,
    secret_token: Option<impl Into<Box<str>>>,
) -> simple::HandlerResult {
    bot.send(
        SetWebhook::new(format!("{webhook_url}{webhook_path}"))
            .allowed_updates(vec![
                UpdateType::Message,
                UpdateType::CallbackQuery,
                UpdateType::MyChatMember,
                UpdateType::ChatJoinRequest,
            ])
            .secret_token_option(secret_token),
    )
    .await?;
    Ok(())
}

fn load_middleware(router: Router) -> Router {
    router.on_update(|observer| {
        observer
            .register_outer_middleware(middlewares::media_group::MediaGroupMiddleware)
            .register_outer_middleware(middlewares::user_update::UpdateUserMiddleware)
            .register_outer_middleware(middlewares::antispam::AntispamMiddleware)
    })
}
pub async fn start(cfg: &Config, db: DatabaseConnection) -> anyhow::Result<()> {
    let bot = Bot::new(cfg.bot_token.to_string());
    let mut main_router = register_routers();

    main_router = load_middleware(main_router);

    // let _ = bot
    //     .send(DeleteWebhook::new().drop_pending_updates(true))
    //     .await;

    let webhook_url = cfg.webhook_url.clone();
    let webhook_path = cfg.webhook_path.clone();
    let secret_token = cfg.secret_token.clone();

    main_router = main_router.on_startup(|observer| {
        observer.register(simple::Handler::new(
            set_webhook,
            (
                bot.clone(),
                webhook_url,
                webhook_path.clone(),
                Some(secret_token.clone()),
            ),
        ))
    });

    let dispatcher = Dispatcher::builder()
        .main_router(main_router.configure_default())
        .bot(bot.clone())
        .extension(db.clone())
        .build();

    let app = AxumRouter::new().route(
        webhook_path.as_ref(),
        get_updates_router(UpdatesHandler::new(bot, dispatcher.clone()).secret_token(secret_token)),
    );

    let (shutdown_tx, _) = channel(1);

    let cfg_clone = cfg.clone();
    let _ = tokio::join!(
        tokio::spawn(run_server(app, shutdown_tx.subscribe(), cfg_clone)),
        tokio::spawn(run_dispatcher(dispatcher, shutdown_tx.subscribe())),
        tokio::spawn(handle_shutdown(shutdown_tx)),
        tokio::spawn(verbal_warns_clear_task(db))
    );

    Ok(())
}

async fn run_server(app: AxumRouter, mut shutdown_rx: Receiver<()>, cfg: Config) {
    let server_host = cfg.server_host.clone();
    let server_port = cfg.server_port;

    let listener = TcpListener::bind(format!("{server_host}:{server_port}"))
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        .unwrap();
}

async fn run_dispatcher(dispatcher: Dispatcher, mut shutdown_rx: Receiver<()>) {
    dispatcher
        .run_no_polling()
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        .unwrap();
}

async fn handle_shutdown(shutdown_tx: Sender<()>) {
    let () = telers::utils::shutdown_signal().await;
    let _ = shutdown_tx.send(());
}
