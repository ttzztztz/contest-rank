use crate::model::{
    render::{Contest, RenderObject, User, UserAggregate},
    website::WebsiteContest,
};
use std::collections::HashMap;

pub fn convert_website_object(
    website_contests: Vec<WebsiteContest>,
    is_live: bool,
) -> RenderObject {
    let mut data = Vec::<Contest>::new();
    let mut aggregate = Vec::<UserAggregate>::new();
    let mut user_hashtable = HashMap::<String, usize>::new();

    for web_contest in website_contests.iter() {
        let mut players = Vec::<User>::new();

        for player in web_contest.players.iter() {
            players.push(User {
                username: player.username.clone(),
                country: player.country.clone(),
                finish_time: player.finish_time,
                global_rank: player.global_rank,
                score: player.score,
                submissions: player.submissions.to_vec(),
                local_rank: 0,
            });

            match user_hashtable.get(&player.username) {
                None => {
                    let aggregate_obj = UserAggregate {
                        username: player.username.clone(),
                        country: player.country.clone(),
                        win_count: 0,
                        attend_count: 1,
                        total_score: player.score,
                        total_time: player.finish_time,
                    };

                    aggregate.push(aggregate_obj);
                    user_hashtable.insert(player.username.clone(), aggregate.len() - 1);
                }
                Some(uid) => {
                    let mut user = &mut (aggregate[*uid]);
                    user.total_score = user.total_score + player.score;
                    user.attend_count += 1;
                    user.total_time += player.finish_time;
                }
            }
        }

        players.sort_by(|lhs, rhs| {
            if lhs.score == rhs.score {
                return lhs.finish_time.cmp(&rhs.finish_time);
            }

            return rhs.score.cmp(&lhs.score);
        });

        for i in 0..players.len() {
            players[i].local_rank = (i + 1) as u32;
        }

        // Winner Count
        if players.len() > 0 {
            let winner_username = players[0].username.clone();
            let uid = user_hashtable.get(&winner_username).unwrap();
            aggregate[*uid].win_count += 1;
        }

        data.push(Contest {
            name: web_contest.name.clone(),
            date: web_contest.date.clone(),
            players,
        });
    }

    aggregate.sort_by(|lhs, rhs| {
        if lhs.win_count == rhs.win_count {
            return rhs.total_score.cmp(&lhs.total_score);
        }
        return rhs.win_count.cmp(&lhs.win_count);
    });

    return RenderObject {
        data,
        aggregate,
        is_live,
    };
}
