use crate::model::render;

fn render_medal(local_rank: u32) -> String {
    if local_rank == 1 {
        return String::from("🏅️");
    } else if local_rank == 2 {
        return String::from("🥈");
    } else if local_rank == 3 {
        return String::from("🥉");
    } else {
        return String::from("");
    }
}

fn render_fail_count(fail_count: u32) -> String {
    if fail_count == 0 {
        return String::from("");
    } else {
        return String::from(format!("🐛 {}", fail_count));
    }
}

pub fn render(object: render::RenderObject) {
    // render for each contest
    for contest in object.data.iter() {
        println!("🏆 {:<48} {}", contest.name, contest.date);
        for player in contest.players.iter() {
            let render_username =
                player.username.clone() + render_medal(player.local_rank).as_str();

            println!(
                "  👴 {:<24} ⏰ {:<12} 📊 {:<4} ✨ {:<6}",
                render_username, player.finish_time, player.global_rank, player.score
            );

            for tid in 0..player.submissions.len() {
                let submission = &player.submissions[tid];

                match submission.status {
                    render::SubmissionStatus::Accepted => {
                        println!(
                            "    ✅ T{} ⏰ {:<12} {}",
                            tid,
                            submission.finish_time,
                            render_fail_count(submission.fail_count)
                        );
                    }
                    render::SubmissionStatus::Unaccepted => {
                        println!("    ❌ T{}", tid);
                    }
                }
            }
        }
    }

    // render aggregate data
    for aggregate in object.aggregate.iter() {
        println!(
            " 👴 {:<24} ✨ {:<6} 🏅️ {:<3} 📊 {:<3}",
            aggregate.username, aggregate.total_score, aggregate.win_count, aggregate.attend_count
        );
    }
}
