using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.UI;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.Foundation;
using Windows.Graphics;
using WinUIEx;
using WinUIEx.Messaging;
using WinUIExSample.Pages;

namespace WinUIExSample
{
    public sealed partial class MainWindow : WindowEx
    {
        internal Queue<string> WindowEvents { get; } = new Queue<string>(101);
        private readonly WindowMessageMonitor monitor;
        private LogWindow logWindow;

        public MainWindow()
        {
            this.InitializeComponent();
            this.SystemBackdrop = WindowDesign.micaBackdrop;
            ExtendsContentIntoTitleBar = true;
            this.SetTitleBarBackgroundColors(Microsoft.UI.Colors.Transparent);
            PersistenceId = "MainWindow";
            monitor = new WindowMessageMonitor(this);
            navigationView.Loaded += NavigationView_Loaded;
            this.Closed += MainWindow_Closed;
        }

        private void MainWindow_Closed(object sender, WindowEventArgs args) => logWindow?.Close();

        private void NavigationView_Loaded(object sender, RoutedEventArgs e)
        {
            navigationView.SelectedItem = navigationView.MenuItems[0];
        }

        private void NavigationView_SelectionChanged(Microsoft.UI.Xaml.Controls.NavigationView sender, Microsoft.UI.Xaml.Controls.NavigationViewSelectionChangedEventArgs args)
        {
            if (args.IsSettingsSelected)
            {
                contentFrame.Navigate(typeof(Pages.Settings));
            }
            else
            {
                var selectedItem = (Microsoft.UI.Xaml.Controls.NavigationViewItem)args.SelectedItem;
                if (selectedItem != null)
                {
                    string selectedItemTag = ((string)selectedItem.Tag);
                    sender.Header = selectedItem.Content;
                    string pageName = "WinUIExSample.Pages." + selectedItemTag;
                    Type pageType = Type.GetType(pageName);
                    contentFrame.Navigate(pageType);
                }
            }
            sender.IsBackEnabled = contentFrame.CanGoBack;
        }

        protected override void OnPositionChanged(PointInt32 position) => Log($"Position Changed: {position.X},{position.Y}");

        protected override void OnPresenterChanged(AppWindowPresenter newPresenter) => Log($"Presenter Changed: {newPresenter.Kind}");

        protected override bool OnSizeChanged(Size size)
        {
            Log($"Size Changed: {size.Width} x {size.Height}");
            return base.OnSizeChanged(size);
        }

        public void Log(string message)
        {
            if (!DispatcherQueue.HasThreadAccess)
            {
                DispatcherQueue.TryEnqueue(() =>
                {
                    Log(message);
                });
                return;
            }
            WindowEvents.Enqueue(message);
            if (WindowEvents.Count > 100)
                WindowEvents.Dequeue();
            logWindow?.UpdateLog();
        }

        protected override void OnStateChanged(WindowState state)
        {
            Log("State changed: " + state);
            base.OnStateChanged(state);
        }

        public void ShowLogWindow()
        {
            if (logWindow is null || logWindow.AppWindow is null)
            {
                logWindow = new LogWindow();
                logWindow.Closed += (s,e) => this.logWindow = null;
            }
            logWindow.Activate();
        }
        

        public void ToggleWMMessages(bool isOn)
        {
            if (isOn)
                monitor.WindowMessageReceived += WindowMessageReceived;
            else
                monitor.WindowMessageReceived -= WindowMessageReceived;
        }

        private void WindowMessageReceived(object sender, WindowMessageEventArgs e)
        {
            Log(e.Message.ToString());
            if(e.Message.MessageId == 0x0005) //WM_SIZE
            {
                // https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-size
                switch (e.Message.WParam)
                {
                    case 0: Debug.WriteLine("Restored" + e.Message.LParam); break;
                    case 1: Debug.WriteLine("Minimized"); break;
                    case 2: Debug.WriteLine("Maximized"); break;
                    case 3: Debug.WriteLine("Max-show"); break;
                    case 4: Debug.WriteLine("Max-hide"); break;
                }
            }
        }

        private void NavigationView_BackRequested(NavigationView sender, NavigationViewBackRequestedEventArgs args)
        {
            if(contentFrame.CanGoBack)
                contentFrame.GoBack();
        }
    }
}