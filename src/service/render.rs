use crate::{model::render, utils::finish_time};
use chrono::{prelude, TimeZone};

fn render_medal(local_rank: u32) -> &'static str {
    match local_rank {
        1 => "🏅️",
        2 => "🥈",
        3 => "🥉",
        _ => "👴",
    }
}

fn render_fail_count(fail_count: u32) -> String {
    match fail_count {
        0 => String::from("🌟BUG FREE"),
        _ => format!("🐛{}", fail_count),
    }
}

fn render_ak(player: &render::User) -> &'static str {
    let mut is_ak = true;
    let mut is_all_bug_free = true;

    for submission in player.submissions.iter() {
        if submission.status != render::SubmissionStatus::Accepted {
            is_ak = false;
            is_all_bug_free = false;

            break;
        }

        if submission.status == render::SubmissionStatus::Accepted && submission.fail_count >= 1 {
            is_all_bug_free = false;
        }
    }

    if is_ak && is_all_bug_free {
        return "🐂AK";
    } else if is_ak {
        return "🎉AK";
    } else {
        return "";
    }
}

pub fn render_date(date: chrono::DateTime<chrono::Local>) -> String {
    return date
        .format_localized("%Y-%m-%d %a %H:%M:%S", prelude::Locale::ja_JP)
        .to_string();
}

pub fn render(object: render::RenderObject, hide_submission: bool) {
    // render for each contest
    if object.is_live {
        println!("🎦[Live] Updated {}", render_date(prelude::Local::now()));
    }

    for contest in object.data.iter() {
        println!(
            "🏆{:<42}{}",
            contest.name,
            render_date(prelude::Local.timestamp(contest.date, 0))
        );

        for player in contest.players.iter() {
            println!(
                "  {}{:<24} 🍺{:<12} 📊{:<4} ✨{:<6} {}",
                render_medal(player.local_rank),
                player.username,
                finish_time::seconds_to_finish_time(player.finish_time),
                player.global_rank,
                player.score,
                render_ak(player)
            );

            if !hide_submission {
                for tid in 0..player.submissions.len() {
                    let submission = &player.submissions[tid];

                    match submission.status {
                        render::SubmissionStatus::Accepted => {
                            println!(
                                "    ✅{:<6} ✨{:<13} ⏰{:<12} {}",
                                submission.title,
                                submission.score,
                                finish_time::seconds_to_finish_time(submission.finish_time),
                                render_fail_count(submission.fail_count)
                            );
                        }
                        render::SubmissionStatus::Unaccepted => {
                            println!("    ❌{:<6} ✨{:<14}", submission.title, submission.score);
                        }
                        render::SubmissionStatus::Pending => {
                            println!("    ⏳{:<6} ✨{:<14}", submission.title, submission.score);
                        }
                    }
                }
            }
        }
        println!("");
    }

    if !object.is_live && object.data.len() >= 2 && !object.aggregate.is_empty() {
        println!("🍎Overall Data");
        // render aggregate data
        for idx in 0..object.aggregate.len() {
            let aggregate = &object.aggregate[idx];
            println!(
                "  {}{:<24} ✨{:<6} 🏅️{:<3} ⚡️{:<4} ⏰{}",
                render_medal(1u32 + (idx as u32)),
                aggregate.username,
                aggregate.total_score,
                aggregate.win_count,
                aggregate.attend_count,
                finish_time::seconds_to_finish_time(aggregate.total_time)
            );
        }
    }
}
