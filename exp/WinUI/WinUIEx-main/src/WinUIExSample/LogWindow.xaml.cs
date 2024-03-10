using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Foundation;
using Windows.Foundation.Collections;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;
using WinUIEx;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace WinUIExSample
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class LogWindow : WinUIEx.WindowEx
    {
        public LogWindow()
        {
            this.InitializeComponent();
            ExtendsContentIntoTitleBar = true;
            UpdateLog();
        }

        public MainWindow MainWindow => (MainWindow)((App)Application.Current).MainWindow;

        internal void UpdateLog()
        {
            var log = MainWindow.WindowEvents;
            if (log != null)
                WindowEventLog.Text = string.Join('\n', log.Reverse());
        }
    }
}
