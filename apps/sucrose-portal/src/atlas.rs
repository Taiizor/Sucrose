//! Releasing images from GPUI's sprite atlas.
//!
//! `App::drop_image` removes an image from the sprite atlas of every window found in `App.windows`. GPUI takes the
//! window being updated out of that map for the duration of its update (event handlers, render, observers of that
//! window), and its doc comment says so: "If the current window is being updated, it will be removed from
//! `App.windows`, you can use `current_window` to specify the current window." A release made with no window from
//! inside such an update therefore silently misses that window and the image keeps its atlas tile, which in turn
//! keeps the whole atlas texture alive (see `docs/GPUI.md`). Every release of the Portal goes through this module,
//! which queues the drop with `App::defer` so it runs after the outermost update, when every window is back.

use std::sync::Arc;

use gpui_kit::{App, RenderImage};

/// Removes `image` from the atlas of every window once the current update has ended.
pub fn release_image(cx: &mut App, image: Arc<RenderImage>) {
    cx.defer(move |cx| cx.drop_image(image, None));
}

/// Removes every image of `images` from the atlas of every window once the current update has ended.
pub fn release_images(cx: &mut App, images: impl IntoIterator<Item = Arc<RenderImage>>) {
    let images: Vec<Arc<RenderImage>> = images.into_iter().collect();
    if images.is_empty() {
        return;
    }
    cx.defer(move |cx| {
        for image in images {
            cx.drop_image(image, None);
        }
    });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui_kit::{
        App, ImageSource, IntoElement as _, ParentElement as _, RenderImage, Styled as _, TestAppContext,
        VisualTestContext, div, img, point, px, size,
    };
    use image::{Frame, ImageBuffer, Rgba};

    // Not `use super::*`: the crate root glob-imports GPUI, whose `test` attribute would shadow the built-in one.
    use super::{release_image, release_images};

    fn test_image() -> Arc<RenderImage> {
        let frame = Frame::new(ImageBuffer::from_pixel(4, 4, Rgba([10, 20, 30, 255])));
        Arc::new(RenderImage::new(vec![frame]))
    }

    /// Opens a window and draws `image` in it so that it enters that window's atlas.
    fn window_showing<'a>(cx: &'a mut TestAppContext, image: &Arc<RenderImage>) -> &'a mut VisualTestContext {
        let window = cx.add_empty_window();
        window.draw(point(px(0.), px(0.)), size(px(40.), px(40.)), |_, _| {
            img(ImageSource::Render(image.clone())).size_full().into_any_element()
        });
        window.update(|window, _| assert!(window.has_image_atlas_entry(image), "the drawn image is in the atlas"));
        window
    }

    fn in_atlas(window: &mut VisualTestContext, image: &Arc<RenderImage>) -> bool {
        window.update(|window, _| window.has_image_atlas_entry(image))
    }

    #[gpui_kit::test]
    fn gpui_drop_image_during_a_window_update_misses_that_window(cx: &mut TestAppContext) {
        let image = test_image();
        let window = window_showing(cx, &image);
        // Recorded cause: App::drop_image with no window cannot reach the window that is being updated.
        window.update(|window, cx| {
            cx.drop_image(image.clone(), None);
            assert!(
                window.has_image_atlas_entry(&image),
                "GPUI left the entry in the updated window"
            );
        });
        window.run_until_parked();
        assert!(in_atlas(window, &image), "the entry is still there after the update");
    }

    #[gpui_kit::test]
    fn release_image_during_a_window_update_empties_the_atlas_entry(cx: &mut TestAppContext) {
        let image = test_image();
        let window = window_showing(cx, &image);
        window.update(|_, cx| release_image(cx, image.clone()));
        window.run_until_parked();
        assert!(!in_atlas(window, &image));
    }

    #[gpui_kit::test]
    fn release_image_outside_a_window_update_empties_the_atlas_entry(cx: &mut TestAppContext) {
        let image = test_image();
        let app = cx.clone();
        let window = window_showing(cx, &image);
        app.update(|cx: &mut App| release_image(cx, image.clone()));
        app.run_until_parked();
        assert!(!in_atlas(window, &image));
    }

    #[gpui_kit::test]
    fn releasing_an_image_twice_is_harmless(cx: &mut TestAppContext) {
        let image = test_image();
        let window = window_showing(cx, &image);
        window.update(|_, cx| {
            release_image(cx, image.clone());
            release_image(cx, image.clone());
        });
        window.run_until_parked();
        assert!(!in_atlas(window, &image));
        window.update(|_, cx| release_images(cx, [image.clone(), image.clone()]));
        window.run_until_parked();
        assert!(!in_atlas(window, &image));
    }

    #[gpui_kit::test]
    fn release_from_an_async_task_and_from_a_window_update_both_empty_the_atlas(cx: &mut TestAppContext) {
        let first = test_image();
        let second = test_image();
        let window = cx.add_empty_window();
        window.draw(point(px(0.), px(0.)), size(px(80.), px(40.)), |_, _| {
            div()
                .child(img(ImageSource::Render(first.clone())).size(px(40.)))
                .child(img(ImageSource::Render(second.clone())).size(px(40.)))
                .into_any_element()
        });
        assert!(in_atlas(window, &first) && in_atlas(window, &second));
        let from_task = first.clone();
        window.update(|_, cx| {
            cx.spawn(async move |cx| cx.update(|cx| release_image(cx, from_task)))
                .detach();
            release_image(cx, second.clone());
        });
        window.run_until_parked();
        assert!(!in_atlas(window, &first), "the release from the async task");
        assert!(!in_atlas(window, &second), "the release from the window update");
    }
}
