use std::time::Duration;

const PARENTS_LIMIT: usize = 10;
const CHILDREN_LIMIT: usize = 5;

const NAME_WIDTH_PARENT: usize = 30;
const NAME_WIDTH_CHILD: usize = 30;
const TIME_WIDTH: usize = 4;

fn fmt_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn cut(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

pub fn format_grouped_lines(items: Vec<(&'static str, Duration, Duration)>) -> (String, Duration) {
    use std::collections::HashMap;

    let mut grouped: HashMap<&str, Vec<(&'static str, Duration, Duration)>> = HashMap::new();

    for (name, last, avg) in items {
        let root = name.split("::").next().unwrap();
        grouped.entry(root).or_default().push((name, last, avg));
    }

    let mut roots: Vec<(&str, Duration)> = grouped
        .iter()
        .map(|(root, v)| {
            let parent_last = v
                .iter()
                .find(|(name, _, _)| *name == *root)
                .map(|(_, last, _)| *last)
                .unwrap_or(Duration::ZERO);
            (*root, parent_last)
        })
        .collect();

    roots.sort_by(|a, b| b.1.cmp(&a.1));
    let roots: Vec<_> = roots.into_iter().take(PARENTS_LIMIT).collect();

    let total_parent_time: Duration = roots.iter().map(|(_, d)| *d).sum();

    let mut lines = String::new();

    for (i, (root, parent_last)) in roots.iter().enumerate() {
        let parts = &grouped[root];

        let parent_avg = parts
            .iter()
            .find(|(name, _, _)| *name == *root)
            .map(|(_, _, avg)| *avg)
            .unwrap_or(Duration::ZERO);

        let parent_percent = if total_parent_time.as_nanos() > 0 {
            parent_last.as_secs_f64() / total_parent_time.as_secs_f64() * 100.0
        } else {
            0.0
        };

        lines.push_str(&format!(
            "  - &a{name:<NAME_WIDTH_PARENT$}&r - &a{last:>TIME_WIDTH$.1}ms&r &2{p:>3.0}% &7(avg {avg:.1}ms)&r",
            name = cut(root, NAME_WIDTH_PARENT),
            last = fmt_ms(*parent_last),
            p = parent_percent,
            avg = fmt_ms(parent_avg),
        ));

        let mut children: Vec<_> = parts.iter().filter(|(name, _, _)| *name != *root).collect();

        children.sort_by(|a, b| b.1.cmp(&a.1));
        let children = children.into_iter().take(CHILDREN_LIMIT).collect::<Vec<_>>();

        let children_total_last: Duration = children.iter().map(|(_, last, _)| *last).sum();

        for (name, last, avg) in children {
            let percent = if children_total_last.as_nanos() > 0 {
                last.as_secs_f64() / children_total_last.as_secs_f64() * 100.0
            } else {
                0.0
            };

            let sub_name = name.split("::").last().unwrap();

            lines.push('\n');
            lines.push_str(&format!(
                "    > &e{name:<NAME_WIDTH_CHILD$}&r > &8{last:>TIME_WIDTH$.1}ms&r {p:>3.0}% &7(avg {avg:.1}ms)&r",
                name = cut(sub_name, NAME_WIDTH_CHILD),
                last = fmt_ms(*last),
                p = percent,
                avg = fmt_ms(*avg),
            ));
        }

        if i + 1 < roots.len() {
            lines.push('\n');
        }
    }

    (lines, total_parent_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn grouped_lines_calculates_total_parent_time() {
        let items = vec![
            // name - avg - last
            ("a", Duration::from_millis(15), Duration::from_millis(10)),
            ("a::child1", Duration::from_millis(6), Duration::from_millis(4)),
            ("b", Duration::from_millis(30), Duration::from_millis(20)),
            ("b::child1", Duration::from_millis(10), Duration::from_millis(6)),
            ("c", Duration::from_millis(15), Duration::from_millis(10)),
            ("c::child1", Duration::from_millis(6), Duration::from_millis(4)),
        ];

        let (text, total) = format_grouped_lines(items);

        assert!(total > Duration::ZERO, "total_parent_time must be > 0");
        assert_eq!(total, Duration::from_millis(60));

        assert_eq!(
            text,
            "  - &ab                             &r - &a30.0ms&r &2 50% &7(avg 20.0ms)&r
    > &echild1                        &r > &810.0ms&r 100% &7(avg 6.0ms)&r
  - &ac                             &r - &a15.0ms&r &2 25% &7(avg 10.0ms)&r
    > &echild1                        &r > &8 6.0ms&r 100% &7(avg 4.0ms)&r
  - &aa                             &r - &a15.0ms&r &2 25% &7(avg 10.0ms)&r
    > &echild1                        &r > &8 6.0ms&r 100% &7(avg 4.0ms)&r",
        );
    }
}
