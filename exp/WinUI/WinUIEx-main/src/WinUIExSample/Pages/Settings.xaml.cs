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
using WinUIEx;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace WinUIExSample.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class Settings : Page
    {
        bool isInitialized;
        public Settings()
        {
            this.InitializeComponent();
           // isInitialized = true;
           // themeSelector.SelectedIndex = ((FrameworkElement)CurrentWindow.Content).RequestedTheme switch { ElementTheme.Dark => 1, ElementTheme.Light => 2, _ => 0 };
        }

        public WindowEx CurrentWindow => ((App)Application.Current).MainWindow;

        public int ThemeSelectedIndex
        {
            get => ((FrameworkElement)CurrentWindow.Content).RequestedTheme switch { ElementTheme.Dark => 1, ElementTheme.Light => 2, _ => 0 };
            set => ((FrameworkElement)CurrentWindow.Content).RequestedTheme = value switch
                {
                    1 => ElementTheme.Dark,
                    2 => ElementTheme.Light,
                    _ => ElementTheme.Default
                };
        }
    }
}
