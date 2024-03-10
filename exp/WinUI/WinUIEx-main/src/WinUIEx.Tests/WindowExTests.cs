namespace WinUIUnitTests;

[TestClass]
[TestCategory(nameof(WinUIEx.WindowEx))]
public partial class WindowExTests
{
    [WorkItem(16)]
    [TestMethod]
    public async Task DefaultContentAlignment()
    {
        await UITestHelper.RunWindowExTest(async (window) =>
        {
            var layoutRoot = new StackPanel() { HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center };
            Button button = new Button() { Content = "Click Me" };
            layoutRoot.Children.Add(button);
            window.WindowContent = layoutRoot;
            await button.LayoutUpdatedAsync();
            var root = window.Content;
            var transform = button.TransformToVisual(root);
            var center = transform.TransformPoint(new Windows.Foundation.Point(button.ActualWidth / 2, button.ActualHeight / 2));
            var rootSize = root.ActualSize;
            rootSize = window.Content.ActualSize;
            Assert.IsTrue(rootSize.X > button.ActualSize.X, "X");
            Assert.IsTrue(rootSize.Y > button.ActualSize.Y, "Y");
            Assert.AreEqual(rootSize.X / 2, center.X, 2, "Center X");
            Assert.AreEqual(rootSize.Y / 2, center.Y, 2, "Center Y");
        });
    }
}