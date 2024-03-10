using Microsoft.UI.Windowing;

namespace WinUIUnitTests
{
    [TestClass]
    [TestCategory(nameof(WinUIEx.WindowManager))]
    public class WindowManagerTests
    {
        public TestContext TestContext { get; set; }

        [TestMethod]
        public async Task GetManagerAlwaysSameInstance()
        {
            await UITestHelper.RunWindowTest((window) =>
            {
                var manager1 = WindowManager.Get(window);
                var manager2 = WindowManager.Get(window);
                Assert.AreSame(manager1, manager2);
            });
        }

        [TestMethod]
        public async Task GetAppWindow()
        {
            await UITestHelper.RunWindowTest((window) =>
            {
                var manager = WindowManager.Get(window);
                Assert.IsNotNull(manager.AppWindow);
                Assert.AreSame(manager.AppWindow, window.AppWindow);
            });
        }

        [TestMethod]
        public async Task SetWidth()
        {
            await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                manager.Width = 500;
                await window.Content.LoadAsync();
                Assert.AreEqual(500, window.AppWindow.Size.Width / window.Content.XamlRoot.RasterizationScale);
            });
        }

        [TestMethod]
        public async Task SetHeight()
        {
            await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                manager.Height = 500;
                await window.Content.LoadAsync();
                Assert.AreEqual(500, window.AppWindow.Size.Height / window.Content.XamlRoot.RasterizationScale);
            });
        }

        [TestMethod]
        public async Task SetMinWidth()
        {
            await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                manager.Width = 500;
                await window.Content.LoadAsync();
                manager.MinWidth = 600;
                Assert.AreEqual(600, window.AppWindow.Size.Width / window.Content.XamlRoot.RasterizationScale);
            });
        }

        [TestMethod]
        public async Task SetMinHeight()
        {
            await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                manager.Height = 500;
                await window.Content.LoadAsync();
                manager.MinHeight = 600;
                Assert.AreEqual(600, window.AppWindow.Size.Height / window.Content.XamlRoot.RasterizationScale);
            });
        }

        [TestMethod]
        public async Task SetMaxWidth() => 
            await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                manager.Width = 600;
                manager.MaxWidth = 500;
                await window.Content.LoadAsync();
                Assert.AreEqual(500, window.AppWindow.Size.Width / window.Content.XamlRoot.RasterizationScale);
            });

        [TestMethod]
        public async Task ResetMaxWidth() =>
            await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                manager.Width = 600;
                manager.MaxWidth = 500;
                await window.Content.LoadAsync();
                Assert.AreEqual(500, window.AppWindow.Size.Width / window.Content.XamlRoot.RasterizationScale);
                manager.MaxWidth = 0;
                manager.Width = 600;
                Assert.AreEqual(600, window.AppWindow.Size.Width / window.Content.XamlRoot.RasterizationScale);
            });

        [TestMethod]
        public async Task SetMaxHeight() => await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                manager.Height = 600;
                manager.MaxHeight = 500;
                await window.Content.LoadAsync();
                Assert.AreEqual(500, window.AppWindow.Size.Height / window.Content.XamlRoot.RasterizationScale);
            });


        [TestMethod]
        public async Task ResetMaxHeight() =>
            await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                manager.Height = 600;
                manager.MaxHeight = 500;
                await window.Content.LoadAsync();
                Assert.AreEqual(500, window.AppWindow.Size.Height / window.Content.XamlRoot.RasterizationScale);
                manager.MaxHeight = 0;
                manager.Height = 600;
                Assert.AreEqual(600, window.AppWindow.Size.Height / window.Content.XamlRoot.RasterizationScale);
            });

        [TestMethod]
        [Timeout(5000)]
        public async Task GetWindowState() => await UITestHelper.RunWindowTest(async (window) =>
            {
                window.Content = new Grid();
                var manager = WindowManager.Get(window);
                await window.Content.LoadAsync();
                WindowState state = manager.WindowState;
                manager.WindowStateChanged += (s, e) =>
                {
                    state = manager.WindowState;
                };
                Assert.AreEqual(WindowState.Normal, manager.WindowState);

                ((OverlappedPresenter)window.AppWindow.Presenter).Maximize();
                Assert.AreEqual(WindowState.Maximized, manager.WindowState);
                Assert.AreEqual(WindowState.Maximized, state);

                ((OverlappedPresenter)window.AppWindow.Presenter).Restore();
                Assert.AreEqual(WindowState.Normal, manager.WindowState);
                Assert.AreEqual(WindowState.Normal, state);

                ((OverlappedPresenter)window.AppWindow.Presenter).Minimize();
                Assert.AreEqual(WindowState.Minimized, manager.WindowState);
                Assert.AreEqual(WindowState.Minimized, state);

                ((OverlappedPresenter)window.AppWindow.Presenter).Restore();
                Assert.AreEqual(WindowState.Normal, manager.WindowState);
                Assert.AreEqual(WindowState.Normal, state);
            });



        [TestMethod]
        [Timeout(5000)]
        public async Task SetWindowState() => await UITestHelper.RunWindowTest(async (window) =>
        {
            window.Content = new Grid();
            var manager = WindowManager.Get(window);
            await window.Content.LoadAsync();
            WindowState state = manager.WindowState;
            manager.WindowStateChanged += (s, e) =>
            {
                state = manager.WindowState;
            };
            Assert.AreEqual(WindowState.Normal, manager.WindowState);

            manager.WindowState = WindowState.Maximized;
            Assert.AreEqual(WindowState.Maximized, state);
            Assert.AreEqual(OverlappedPresenterState.Maximized, ((OverlappedPresenter)window.AppWindow.Presenter).State);

            manager.WindowState = WindowState.Normal;
            Assert.AreEqual(WindowState.Normal, state);
            Assert.AreEqual(OverlappedPresenterState.Restored, ((OverlappedPresenter)window.AppWindow.Presenter).State);

            manager.WindowState = WindowState.Minimized;
            Assert.AreEqual(WindowState.Minimized, state);
            Assert.AreEqual(OverlappedPresenterState.Minimized, ((OverlappedPresenter)window.AppWindow.Presenter).State);

            manager.WindowState = WindowState.Normal;
            Assert.AreEqual(WindowState.Normal, state);
            Assert.AreEqual(OverlappedPresenterState.Restored, ((OverlappedPresenter)window.AppWindow.Presenter).State);
        });

        [TestMethod]
        [Timeout(5000)]
        public async Task PersistenceId() => await UITestHelper.RunWindowTest(async (window) =>
        {
            window.Content = new Grid();
            var manager = WindowManager.Get(window);
            manager.PersistenceId = "PersistenceId";
            await window.Content.LoadAsync();
            manager.Width = 456;
            manager.Height = 345;
            window.AppWindow.MoveAndResize(new Windows.Graphics.RectInt32(45, 67, 456, 345));
            window.Close();
            Window newWindow = new Window();
            var manager2 = WindowManager.Get(newWindow);
            manager2.PersistenceId = manager.PersistenceId;
            newWindow.Content = new Grid();
            try
            {
                newWindow.Activate();
                await newWindow.Content.LoadAsync();
                var position = newWindow.AppWindow.Position;
                var size = newWindow.AppWindow.Size;
                Assert.AreEqual(45, position.X);
                Assert.AreEqual(67, position.Y);
                Assert.AreEqual(456, size.Width);
                Assert.AreEqual(345, size.Height);
            }
            finally //Ensure window is closed if test fails
            {
                newWindow.Close();
            }
        });
    }

}
