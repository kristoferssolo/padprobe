use crate::{ClusterPlacement, ControlCluster, ControlValue};
use ratatui::layout::Rect;

#[inline]
pub(super) fn can_render_controller(area: Rect, clusters: &[ControlCluster]) -> bool {
    const MIN_WIDTH: u16 = 48;
    const MIN_HEIGHT: u16 = 25;

    area.width >= MIN_WIDTH
        && area.height >= MIN_HEIGHT
        && clusters.iter().enumerate().all(|(index, cluster)| {
            let placement = cluster.placement();
            !matches!(placement, ClusterPlacement::Flow | ClusterPlacement::Extra)
                && cluster_fits_controller_art(cluster)
                && !clusters[..index]
                    .iter()
                    .any(|previous| previous.placement() == placement)
        })
}

#[inline]
fn cluster_fits_controller_art(cluster: &ControlCluster) -> bool {
    match cluster.placement() {
        ClusterPlacement::LeftStick | ClusterPlacement::RightStick => {
            matches!(
                cluster.controls(),
                [control] if matches!(control.value(), ControlValue::Stick { .. })
            )
        }
        ClusterPlacement::DPad | ClusterPlacement::Face => cluster.controls().len() <= 4,
        ClusterPlacement::Flow
        | ClusterPlacement::LeftShoulder
        | ClusterPlacement::Menu
        | ClusterPlacement::RightShoulder
        | ClusterPlacement::Extra => true,
    }
}

pub(super) fn grid_areas(area: Rect, item_count: usize) -> Vec<Rect> {
    let columns = match area.width {
        89.. => item_count.min(3),
        59.. => item_count.min(2),
        _ => 1,
    };
    let rows = item_count.div_ceil(columns);
    let column_gap = usize::from(columns > 1);
    let row_gap = usize::from(rows > 1);
    let usable_width =
        usize::from(area.width).saturating_sub(column_gap * columns.saturating_sub(1));
    let usable_height = usize::from(area.height).saturating_sub(row_gap * rows.saturating_sub(1));
    let column_width = usable_width / columns;
    let row_height = usable_height / rows;

    (0..item_count)
        .map(|index| {
            let column = index % columns;
            let row = index / columns;
            let x = area.x.saturating_add(
                u16::try_from(column * (column_width + column_gap)).unwrap_or(u16::MAX),
            );
            let y = area
                .y
                .saturating_add(u16::try_from(row * (row_height + row_gap)).unwrap_or(u16::MAX));
            let width = if column + 1 == columns {
                area.right().saturating_sub(x)
            } else {
                u16::try_from(column_width).unwrap_or(u16::MAX)
            };
            let height = if row + 1 == rows {
                area.bottom().saturating_sub(y)
            } else {
                u16::try_from(row_height).unwrap_or(u16::MAX)
            };
            Rect::new(x, y, width, height)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Control;

    #[test]
    fn grid_adapts_column_count_to_width() {
        let narrow = grid_areas(Rect::new(0, 0, 30, 30), 6);
        let medium = grid_areas(Rect::new(0, 0, 60, 15), 6);
        let wide = grid_areas(Rect::new(0, 0, 90, 10), 6);

        assert_eq!(narrow[1].x, narrow[0].x);
        assert!(medium[1].x > medium[0].x);
        assert!(wide[2].x > wide[1].x);
    }

    #[test]
    fn semantic_layout_falls_back_in_small_areas() {
        let clusters =
            [ControlCluster::new("Left stick").with_placement(ClusterPlacement::LeftStick)];

        assert!(!can_render_controller(Rect::new(0, 0, 60, 12), &clusters));
    }

    #[test]
    fn semantic_layout_accepts_standard_placements() {
        let clusters = [ControlCluster::new("Menu").with_placement(ClusterPlacement::Menu)];

        assert!(can_render_controller(Rect::new(0, 0, 100, 25), &clusters));
    }

    #[test]
    fn semantic_layout_does_not_clip_extra_cluster() {
        let clusters = [
            ControlCluster::new("Left stick").with_placement(ClusterPlacement::LeftStick),
            ControlCluster::new("Extra").with_placement(ClusterPlacement::Extra),
        ];

        assert!(!can_render_controller(Rect::new(0, 0, 100, 25), &clusters));
    }

    #[test]
    fn semantic_layout_does_not_hide_duplicate_placements() {
        let clusters = [
            ControlCluster::new("First").with_placement(ClusterPlacement::LeftStick),
            ControlCluster::new("Second").with_placement(ClusterPlacement::LeftStick),
        ];

        assert!(!can_render_controller(Rect::new(0, 0, 100, 25), &clusters));
    }

    #[test]
    fn semantic_layout_does_not_hide_malformed_stick_cluster() {
        let clusters = [ControlCluster::new("Left stick")
            .with_placement(ClusterPlacement::LeftStick)
            .with_control(Control::new("unexpected", ControlValue::button(false)))];

        assert!(!can_render_controller(Rect::new(0, 0, 100, 25), &clusters));
    }
}
