mod callbacks;
pub mod enums;
mod filters;
mod handlers;
mod keyboards;
pub mod libs;
mod methods;
pub mod middlewares;
mod task;
mod utils;

use crate::bot::handlers::register_routers;
use crate::bot::task::verbal_warns_clear_task;
use crate::config::Config;
use sea_orm::DatabaseConnection;
use telers::enums::UpdateType;
use telers::{Bot, Dispatcher, Router};
use tokio::sync::broadcast::{Receiver, Sender, channel};

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

    let allowed_updates = [
        UpdateType::Message,
        UpdateType::CallbackQuery,
        UpdateType::ChatMember,
        UpdateType::MyChatMember,
        UpdateType::ChatJoinRequest,
    ];

    let dispatcher = Dispatcher::builder()
        .main_router(main_router.configure_default())
        .allowed_updates(allowed_updates)
        .bot(bot.clone())
        .extension(db.clone())
        .build();

    let (shutdown_tx, _) = channel(1);

    let _ = tokio::join!(
        tokio::spawn(run_dispatcher(dispatcher, shutdown_tx.subscribe())),
        tokio::spawn(handle_shutdown(shutdown_tx)),
        tokio::spawn(verbal_warns_clear_task(db))
    );

    Ok(())
}

async fn run_dispatcher(dispatcher: Dispatcher, mut shutdown_rx: Receiver<()>) {
    dispatcher
        .run_polling()
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
