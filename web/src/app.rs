use leptos::prelude::*;

use crate::api::{self, Achievement, Dashboard, Game, Profile};
#[component]
pub fn App() -> impl IntoView {
    // LocalResource roda no navegador; `.refetch()` refaz a busca.
    let dashboard = LocalResource::new(api::dashboard);

    view! {
        <main class="page">
            <button class="btn" on:click=move |_| dashboard.refetch()>
                "Atualizar"
            </button>

            // Transition mantém o conteúdo antigo na tela enquanto atualiza.
            <Transition fallback=|| view! { <p>"Carregando…"</p> }>
                {move || Suspend::new(async move {
                    match dashboard.await {
                        Ok(data) => view! { <DashboardView data /> }.into_any(),
                        Err(error) => {
                            view! { <p class="error" role="alert">{error.message}</p> }.into_any()
                        }
                    }
                })}
            </Transition>
        </main>
    }
}

#[component]
fn DashboardView(data: Dashboard) -> impl IntoView {
    view! {
        <ProfileView profile=data.profile />
        <h2>"Jogos recentes"</h2>
        <GameList games=data.recent_games />
        <h2>"Conquistas recentes"</h2>
        <AchievementList items=data.recent_achievements />
    }
}

#[component]
fn ProfileView(profile: Profile) -> impl IntoView {
    view! {
        <section class="profile">
            <img class="avatar" src=profile.avatar_url alt="Avatar" width="96" height="96" />
            <div>
                <h1>{profile.user}</h1>
                <p>{profile.motto}</p>
                <p>
                    {format!(
                        "{} pontos ({} RetroPoints)",
                        profile.total_points,
                        profile.total_true_points,
                    )}
                </p>
                <p>{profile.last_activity}</p>
            </div>
        </section>
    }
}

#[component]
fn GameList(games: Vec<Game>) -> impl IntoView {
    if games.is_empty() {
        return view! { <p>"Nenhum jogo ainda."</p> }.into_any();
    }

    view! {
        <ul class="rows">
            {games
                .into_iter()
                .map(|game| {
                    view! {
                        <li class="row">
                            <img src=game.icon_url alt="" width="64" height="64" />
                            <div>
                                <a href=game.game_url target="_blank" rel="noreferrer">
                                    {game.title}
                                </a>
                                <p class="muted">{game.console_name}</p>
                                <progress max="100" value=game.completion_percent.to_string()></progress>
                                <p class="muted">
                                    {format!(
                                        "{} de {} conquistas",
                                        game.achievements_earned,
                                        game.achievements_total,
                                    )}
                                </p>
                            </div>
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
    .into_any()
}

#[component]
fn AchievementList(items: Vec<Achievement>) -> impl IntoView {
    if items.is_empty() {
        return view! { <p>"Nenhuma conquista nos últimos 30 dias."</p> }.into_any();
    }

    view! {
        <ul class="rows">
            {items
                .into_iter()
                .map(|a| {
                    view! {
                        <li class="row">
                            <img src=a.badge_url alt="" width="56" height="56" />
                            <div>
                                <a href=a.achievement_url target="_blank" rel="noreferrer">
                                    {a.title}
                                </a>
                                <p>{a.description}</p>
                                <p class="muted">{a.game_title}</p>
                                <p class="muted">{a.unlocked_at}</p>
                            </div>
                            <span class="chip">{format!("{} pts", a.points)}</span>
                            {a.hardcore.then(|| view! { <span class="chip hardcore">"Hardcore"</span> })}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
    .into_any()
}
