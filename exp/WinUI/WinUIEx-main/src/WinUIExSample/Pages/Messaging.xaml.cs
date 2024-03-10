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
using Windows.Foundation;
using Windows.Foundation.Collections;
using WinUIEx.Messaging;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace WinUIExSample.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class Messaging : Page
    {
        private static WindowMessageMonitor? monitor;
        public Messaging()
        {
            this.InitializeComponent();
        }

        public MainWindow MainWindow => (MainWindow)((App)Application.Current).MainWindow;

        private void WMMessages_Toggled(object sender, RoutedEventArgs e) => MainWindow.ToggleWMMessages(((ToggleSwitch)sender).IsOn);
        private void OpenLogWindow_Click(object sender, RoutedEventArgs e) => MainWindow.ShowLogWindow();
    }
}
