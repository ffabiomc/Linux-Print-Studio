//! Pure page-order planning. No PDF output or printer integration yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetSide {
    /// Source page numbers (1-based), None means a blank slot.
    pub slots: Vec<Option<usize>>,
}

pub fn n_up(pages: usize, per_sheet: usize) -> Result<Vec<SheetSide>, &'static str> {
    if pages == 0 || ![1, 2, 4, 6, 9, 16].contains(&per_sheet) {
        return Err("Invalid page count or N-up layout");
    }
    Ok((1..=pages)
        .collect::<Vec<_>>()
        .chunks(per_sheet)
        .map(|chunk| SheetSide {
            slots: chunk.iter().copied().map(Some).chain(
                std::iter::repeat(None).take(per_sheet - chunk.len()),
            ).collect(),
        })
        .collect())
}

/// Booklet in physical print order: front then back for each sheet.
/// Left and right slots are for a landscape sheet folded in half.
/// Duplex edge/rotation must be configured separately for the printer.
pub fn booklet(pages: usize) -> Result<Vec<SheetSide>, &'static str> {
    if pages == 0 || pages > 100_000 {
        return Err("Invalid page count");
    }
    let padded = pages.checked_add(3).ok_or("Too many pages")? / 4 * 4;
    let slot = |n: usize| if n <= pages { Some(n) } else { None };
    let mut sides = Vec::with_capacity(padded / 2);
    for sheet in 0..(padded / 4) {
        sides.push(SheetSide {
            slots: vec![slot(padded - 2 * sheet), slot(1 + 2 * sheet)],
        });
        sides.push(SheetSide {
            slots: vec![slot(2 + 2 * sheet), slot(padded - 1 - 2 * sheet)],
        });
    }
    Ok(sides)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn booklet_eight_pages() {
        let sides = booklet(8).unwrap();
        assert_eq!(sides[0].slots, vec![Some(8), Some(1)]);
        assert_eq!(sides[1].slots, vec![Some(2), Some(7)]);
        assert_eq!(sides[2].slots, vec![Some(6), Some(3)]);
        assert_eq!(sides[3].slots, vec![Some(4), Some(5)]);
    }

    #[test]
    fn booklet_pads_six_pages() {
        let sides = booklet(6).unwrap();
        assert_eq!(sides[0].slots, vec![None, Some(1)]);
        assert_eq!(sides[1].slots, vec![Some(2), None]);
        assert_eq!(sides.len(), 4);
    }

    #[test]
    fn n_up_pads_last_sheet() {
        let sides = n_up(5, 4).unwrap();
        assert_eq!(sides.len(), 2);
        assert_eq!(sides[1].slots, vec![Some(5), None, None, None]);
    }

    #[test]
    fn invalid_inputs_rejected() {
        assert!(booklet(0).is_err());
        assert!(n_up(2, 3).is_err());
    }
}
