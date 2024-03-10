namespace WinUIUnitTests;

[TestClass]
[TestCategory(nameof(WinUIEx.WindowExtensions))]
public partial class WindowExtensionTests
{
    [TestMethod]
    public async Task SetWidth()
    {
        await UITestHelper.RunWindowExTest(async (WindowContext) =>
        {
            var grid = new Grid();
            WindowContext.Content = grid;
            await grid.SizeChangedAsync();
            var width = grid.ActualWidth;
            var windowPadding = WindowContext.Width - width;
            WindowContext.Width = 500;
            await grid.SizeChangedAsync();
            Assert.AreEqual(500, grid.ActualWidth + windowPadding, 0.01, "Width after window resize");
        });
    }

    [TestMethod]
    public async Task SetHeight()
    {
        await UITestHelper.RunWindowExTest(async (WindowEx) =>
        {
            var grid = new Grid();
            WindowEx.Content = grid;
            await grid.LoadAsync();
            var height = grid.ActualHeight;
            var windowPadding = WindowEx.Height - height;
            WindowEx.Height = 500;
            await grid.SizeChangedAsync();
            Assert.AreEqual(500, grid.ActualHeight + windowPadding, 0.01, "Height after window resize");
        });
    }

    [TestMethod]
    public async Task Minimize()
    {
        await UITestHelper.RunWindowTest((WindowEx) =>
        {
            Assert.IsTrue(WindowEx.Visible);
            WindowEx.Minimize();
            var presenter = WindowEx.AppWindow.Presenter as Microsoft.UI.Windowing.OverlappedPresenter;
            Assert.AreEqual(Microsoft.UI.Windowing.OverlappedPresenterState.Minimized, presenter.State);
            Assert.IsFalse(WindowEx.Visible);
            WindowEx.Restore();
            Assert.AreEqual(Microsoft.UI.Windowing.OverlappedPresenterState.Restored, presenter.State);
            Assert.IsTrue(WindowEx.Visible);
            return Task.CompletedTask;
        });
    }

    [TestMethod]
    public async Task Maximize()
    {
        await UITestHelper.RunWindowTest(async (WindowEx) =>
        {
            var grid = new Grid();
            WindowEx.Content = grid;
            await grid.LoadAsync();
            var size = grid.ActualHeight;
            var presenter = WindowEx.AppWindow.Presenter as Microsoft.UI.Windowing.OverlappedPresenter;
            WindowEx.Maximize();
            Assert.AreEqual(Microsoft.UI.Windowing.OverlappedPresenterState.Maximized, presenter.State);
            await grid.SizeChangedAsync();
            Assert.IsTrue(grid.ActualHeight > size, "grid.ActualHeight > size");
            size = grid.ActualHeight;
            WindowEx.Restore();
            Assert.AreEqual(Microsoft.UI.Windowing.OverlappedPresenterState.Restored, presenter.State);
            await grid.SizeChangedAsync();
            Assert.IsTrue(grid.ActualHeight < size, "grid.ActualHeight < size");
        });
    }

    [TestMethod]
    public async Task CompactPresenter()
    {
        await UITestHelper.RunWindowExTest(async (WindowEx) =>
        {
            var grid = new Grid();
            WindowEx.Content = grid;
            await grid.LoadAsync();
            var width = WindowEx.Bounds.Width;
            var height = WindowEx.Bounds.Height;
            WindowEx.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.CompactOverlay;
            await grid.SizeChangedAsync();
            Assert.IsTrue(grid.ActualWidth < width);
            Assert.IsTrue(grid.ActualHeight < height);
        });
    }

    [TestMethod]
    public async Task FullscreenPresenter()
    {
        await UITestHelper.RunWindowExTest(async (WindowEx) =>
        {
            var grid = new Grid();
            WindowEx.Content = grid;
            await grid.LoadAsync();
            var width = WindowEx.Bounds.Width;
            var height = WindowEx.Bounds.Height;
            WindowEx.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.FullScreen;
            await grid.SizeChangedAsync();
            Assert.IsTrue(grid.ActualWidth > width, "width");
            Assert.IsTrue(grid.ActualHeight > height, "height");
        });
    }
}
