use chrono::prelude;

use crate::model::render;

fn render_medal(local_rank: u32) -> &'static str {
    if local_rank == 1 {
        return "🏅️";
    } else if local_rank == 2 {
        return "🥈";
    } else if local_rank == 3 {
        return "🥉";
    } else {
        return "👴";
    }
}

fn render_fail_count(fail_count: u32) -> String {
    if fail_count == 0 {
        return String::from("🌟 BUG FREE");
    } else {
        return format!("🐛 {}", fail_count);
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

pub fn render(object: render::RenderObject) {
    // render for each contest
    if object.is_live {
        println!(
            "🎦 [Live] Updated {}",
            prelude::Local::now()
                .format_localized("%Y-%m-%d %a %H:%M:%S", prelude::Locale::ja_JP)
                .to_string()
        );
    }

    for contest in object.data.iter() {
        println!("🏆 {:<48} {}", contest.name, contest.date);

        for player in contest.players.iter() {
            println!(
                "  {} {:<24} 🍺{:<12} 📊{:<4} ✨{:<6} {}",
                render_medal(player.local_rank),
                player.username,
                player.finish_time,
                player.global_rank,
                player.score,
                render_ak(player)
            );

            for tid in 0..player.submissions.len() {
                let submission = &player.submissions[tid];

                match submission.status {
                    render::SubmissionStatus::Accepted => {
                        println!(
                            "    ✅{:<6} ✨{:<14} ⏰{:<12} {}",
                            submission.title,
                            submission.score,
                            submission.finish_time,
                            render_fail_count(submission.fail_count)
                        );
                    }
                    render::SubmissionStatus::Unaccepted => {
                        println!("    ❌{:<6} ✨{:<14}", submission.title, submission.score);
                    }
                }
            }
        }
        println!("");
    }

    if !object.is_live {
        println!("🍎 Overall Data");
        // render aggregate data
        for aggregate in object.aggregate.iter() {
            println!(
                "  👴 {:<24} ✨{:<6} 🏅️{:<3} ⚡️{:<3}",
                aggregate.username,
                aggregate.total_score,
                aggregate.win_count,
                aggregate.attend_count
            );
        }
    }
}
