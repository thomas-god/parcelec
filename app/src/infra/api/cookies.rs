use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

use crate::{
    game::GameId,
    player::{PlayerId, PlayerName},
};

pub fn add_game_cookies(
    cookies: &Cookies,
    player: &PlayerId,
    player_name: &PlayerName,
    game: &GameId,
) {
    let player_id_cookie = Cookie::build(("player_id", player.to_string()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(player_id_cookie);
    let game_id_cookie = Cookie::build(("game_id", game.to_string()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(game_id_cookie);
    let name_cookie = Cookie::build(("player_name", player_name.to_string()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(name_cookie);
}

pub fn extract_game_cookies(cookies: &Cookies) -> Option<(PlayerId, PlayerName, GameId)> {
    let id = cookies
        .get("player_id")
        .map(|c| PlayerId::from(c.value_trimmed()))?;
    let name = cookies
        .get("player_name")
        .map(|c| PlayerName::from(c.value_trimmed()))?;
    let game_id = cookies
        .get("game_id")
        .map(|c| GameId::from(c.value_trimmed()))?;
    Some((id, name, game_id))
}

pub fn invalidate_game_cookies(cookies: &Cookies) {
    let game_id_cookie = Cookie::build(("game_id", "".to_string()))
        .max_age(Duration::seconds(0))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(game_id_cookie);
    let player_id_cookie = Cookie::build(("player_id", "".to_string()))
        .max_age(Duration::seconds(0))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(player_id_cookie);
    let name_cookie = Cookie::build(("player_name", "".to_string()))
        .max_age(Duration::seconds(0))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(name_cookie);
}
