use std::time::Duration;

const PARENTS_LIMIT: usize = 10;
const CHILDREN_LIMIT: usize = 5;

const NAME_WIDTH: usize = 30;
const CHILD_PREFIX_DIFF: usize = 3;
const TIME_WIDTH: usize = 10;

macro_rules! format_parent {
    () => {
        "&8├─ &a{name}&r &8│ &2{last}&r &8│ &2{percent:>3.0}%&r &8│ &8avg &2{avg}"
    };
}

macro_rules! format_child {
    () => {
        "&8│  └─ &e{name}&r &8│ &7{last}&r &8│ {percent:>3.0}% &8│ &8avg {avg}"
    };
}

fn fmt_dur(d: Duration) -> String {
    format!("{:>TIME_WIDTH$}", format!("{:.1?}", d))
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

    roots.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
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
            format_parent!(),
            name = format!("{:<NAME_WIDTH$}", cut(root, NAME_WIDTH)),
            last = fmt_dur(*parent_last),
            percent = parent_percent,
            avg = fmt_dur(parent_avg),
        ));

        let mut children: Vec<_> = parts.iter().filter(|(name, _, _)| *name != *root).collect();

        children.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
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
            let child_width = NAME_WIDTH - CHILD_PREFIX_DIFF;
            lines.push_str(&format!(
                format_child!(),
                name = format!("{:<child_width$}", cut(sub_name, child_width)),
                last = fmt_dur(*last),
                percent = percent,
                avg = fmt_dur(*avg),
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
            "&8├─ &ab                             &r &8│ &2    30.0ms&r &8│ &2 50%&r &8│ &8avg &2    20.0ms
&8│  └─ &echild1                     &r &8│ &7    10.0ms&r &8│ 100% &8│ &8avg      6.0ms
&8├─ &aa                             &r &8│ &2    15.0ms&r &8│ &2 25%&r &8│ &8avg &2    10.0ms
&8│  └─ &echild1                     &r &8│ &7     6.0ms&r &8│ 100% &8│ &8avg      4.0ms
&8├─ &ac                             &r &8│ &2    15.0ms&r &8│ &2 25%&r &8│ &8avg &2    10.0ms
&8│  └─ &echild1                     &r &8│ &7     6.0ms&r &8│ 100% &8│ &8avg      4.0ms",
        );
    }
}
