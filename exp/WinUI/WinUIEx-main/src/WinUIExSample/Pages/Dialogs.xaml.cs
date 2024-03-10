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
    public sealed partial class Dialogs : Page
    {
        public Dialogs()
        {
            this.InitializeComponent();
        }

        public WindowEx MainWindow => ((App)Application.Current).MainWindow;

        private async void ShowDialog_Click(object sender, RoutedEventArgs e)
        {
            resultText.Text = "";
            await MainWindow.ShowMessageDialogAsync("Hello World!", "Dialog title");
            resultText.Text = "Dialog closed";
        }

        private async void ShowDialog2_Click(object sender, RoutedEventArgs e)
        {
            resultText.Text = "";
            var commands = new List<Windows.UI.Popups.IUICommand>();
            commands.Add(new Windows.UI.Popups.UICommand("OK"));
            commands.Add(new Windows.UI.Popups.UICommand("Cancel"));
            var result = await MainWindow.ShowMessageDialogAsync("Are you sure?", commands, cancelCommandIndex: 2, title: "Dialog title");
            resultText.Text = "You clicked: " + result.Label;
        }

        private async void ShowDialog3_Click(object sender, RoutedEventArgs e)
        {
            resultText.Text = "";
            var commands = new List<Windows.UI.Popups.IUICommand>
            {
                new Windows.UI.Popups.UICommand("Red Pill"),
                new Windows.UI.Popups.UICommand("Blue Pill"),
                new Windows.UI.Popups.UICommand("Cancel")
            };
            string message = "This is your last chance. After this, there is no turning back. You take the blue pill – the story ends, you wake up in your bed and believe whatever you want to believe. You take the red pill – you stay in Wonderland, and I show you how deep the rabbit hole goes. Remember, all I'm offering is the truth – nothing more.";
            var result = await MainWindow.ShowMessageDialogAsync(message, commands, cancelCommandIndex: 2, title: "Morpheus Asks");
            resultText.Text = "You chose: " + result.Label;
        }

    }
}
