//! Manual duplex planning. The user must test paper orientation on their printer.
//! Input is a sequence of imposed sheet sides, front then back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedOrder {
    Same,
    Reversed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplexPlan {
    /// One-based page numbers in the already imposed PDF.
    pub fronts: Vec<usize>,
    pub backs: Vec<usize>,
}

pub fn plan_manual_duplex(
    side_count: usize,
    back_feed: FeedOrder,
) -> Result<DuplexPlan, &'static str> {
    if side_count == 0 || side_count > 100_000 {
        return Err("Invalid sheet-side count");
    }
    let fronts = (1..=side_count).step_by(2).collect();
    let mut backs: Vec<usize> = (2..=side_count).step_by(2).collect();
    if back_feed == FeedOrder::Reversed {
        backs.reverse();
    }
    Ok(DuplexPlan { fronts, backs })
}

/// Number of physical sheets required for an imposed sequence of sides.
pub fn physical_sheet_count(side_count: usize) -> usize {
    side_count / 2 + side_count % 2
}

/// Whether the final physical sheet has no printed back side.
pub fn final_back_is_blank(side_count: usize) -> bool {
    side_count % 2 != 0
}

pub fn page_range(pages: &[usize]) -> String {
    pages
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_sides_same_order() {
        let plan = plan_manual_duplex(8, FeedOrder::Same).unwrap();
        assert_eq!(plan.fronts, vec![1, 3, 5, 7]);
        assert_eq!(plan.backs, vec![2, 4, 6, 8]);
    }

    #[test]
    fn reversed_backs() {
        let plan = plan_manual_duplex(8, FeedOrder::Reversed).unwrap();
        assert_eq!(plan.backs, vec![8, 6, 4, 2]);
    }

    #[test]
    fn odd_sides_last_back_is_blank() {
        let plan = plan_manual_duplex(5, FeedOrder::Same).unwrap();
        assert_eq!(plan.fronts, vec![1, 3, 5]);
        assert_eq!(plan.backs, vec![2, 4]);
    }

    #[test]
    fn reject_empty() {
        assert!(plan_manual_duplex(0, FeedOrder::Same).is_err());
    }

    #[test]
    fn physical_sheet_counts_and_blank_backs() {
        assert_eq!(physical_sheet_count(1), 1);
        assert_eq!(physical_sheet_count(5), 3);
        assert_eq!(physical_sheet_count(8), 4);
        assert!(final_back_is_blank(5));
        assert!(!final_back_is_blank(8));
    }

    #[test]
    fn page_ranges() {
        assert_eq!(page_range(&[8, 6, 4, 2]), "8,6,4,2");
    }
}
