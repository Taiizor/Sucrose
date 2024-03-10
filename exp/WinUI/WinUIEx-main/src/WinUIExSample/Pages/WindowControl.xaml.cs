using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Text;
using System.Threading.Tasks;
using Windows.Foundation;
using Windows.Foundation.Collections;
using WinUIEx;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace WinUIExSample.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class WindowControl : Page
    {
        public WindowControl()
        {
            this.InitializeComponent();
            windowState.SelectedIndex = (int)MainWindow.WindowState;
            this.Loaded += WindowControl_Loaded;
            this.Unloaded += WindowControl_Unloaded;
        }

        public WindowEx MainWindow => ((App)Application.Current).MainWindow;


        private void WindowControl_Loaded(object sender, RoutedEventArgs e)
        {
            MainWindow.WindowStateChanged += CurrentWindow_WindowStateChanged;
        }

        private void WindowControl_Unloaded(object sender, RoutedEventArgs e)
        {
            MainWindow.WindowStateChanged += CurrentWindow_WindowStateChanged;
        }

        private void CurrentWindow_WindowStateChanged(object sender, WindowState e)
        {
            windowState.SelectedIndex = (int)e;
        }

        private void windowState_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            MainWindow.WindowState = (WindowState)windowState.SelectedIndex;
        }

        private void Center_Click(object sender, RoutedEventArgs e) => MainWindow.CenterOnScreen();

        private void MaximizeWindow_Click(object sender, RoutedEventArgs e) => MainWindow.Maximize();

        private void RestoreWindow_Click(object sender, RoutedEventArgs e) => MainWindow.Restore();

        private async void MinimizeWindow_Click(object sender, RoutedEventArgs e)
        {
            MainWindow.Minimize();
            await Task.Delay(2000);
            MainWindow.Restore();
        }

        private async void HideWindow_Click(object sender, RoutedEventArgs e)
        {
            MainWindow.Hide();
            await Task.Delay(2000);
            MainWindow.Restore();
        }

        private async void BringToFront_Click(object sender, RoutedEventArgs e)
        {
            await Task.Delay(2000);
            MainWindow.BringToFront();
        }


        private void limitMaxCheckbox_Toggled(object sender, RoutedEventArgs e)
        {
            if (limitMaxCheckbox.IsOn)
            {
                sliderMaxWidth.Value = MainWindow.Width;
                sliderMaxHeight.Value = MainWindow.Height;
            }
            else
            {
                MainWindow.MaxWidth = 0;
                MainWindow.MaxHeight = 0;
            }
        }
        public int WindowStateSelectedIndex
        {
            get => (int)MainWindow.WindowState;
            set => MainWindow.WindowState = (WindowState)value;
        }

        private void Slider_ValueChanged(object sender, Microsoft.UI.Xaml.Controls.Primitives.RangeBaseValueChangedEventArgs e)
        {
            MainWindow.SetWindowOpacity((byte)e.NewValue);
        }

        private void DisplayInfo_Click(object sender, RoutedEventArgs e)
        {

            var monitors = MonitorInfo.GetDisplayMonitors();
            StringBuilder sb = new StringBuilder($"{monitors.Count} monitors detected");
            sb.AppendLine();
            foreach (var monitor in monitors)
                sb.AppendLine(monitor.ToString());
            _  = MainWindow.ShowMessageDialogAsync(sb.ToString(), title: "Display Info");
        }
    }
}
