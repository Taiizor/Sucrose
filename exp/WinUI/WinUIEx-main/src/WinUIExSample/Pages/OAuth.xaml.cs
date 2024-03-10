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
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Xml;
using Windows.Foundation;
using Windows.Foundation.Collections;
using Windows.Web.Http;
using WinUIEx;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace WinUIExSample.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class OAuth : Page
    {
        public OAuth()
        {
            this.InitializeComponent();
        }
        public WindowEx MainWindow => ((App)Application.Current).MainWindow;

        private CancellationTokenSource? oauthCancellationSource;
        private void DoOAuth_Click(object sender, RoutedEventArgs e)
        {
            DoOAuth("code");
        }

        private void DoOAuth2_Click(object sender, RoutedEventArgs e)
        {
            DoOAuth("token");
        }
        private async void DoOAuth(string responseType)
        { 
            using MockOAuthServer server = new MockOAuthServer();

            Result.Text = "Waiting for login in browser...";
            string clientId = "imIwo061j9SUOQYm7O8Oe4HK";
            string callbackUri = "winuiex://";
            string authorizeUri = $"{server.Url}?response_type={responseType}&client_id={clientId}&redirect_uri={Uri.EscapeDataString(callbackUri)}&scope=photo+offline_access";
            if(!string.IsNullOrEmpty(stateField.Text))
            {
                authorizeUri += $"&state={Uri.EscapeDataString(stateField.Text)}";
            }
            oauthCancellationSource = new CancellationTokenSource();
            oauthCancellationSource.Token.Register(() => { OAuthWindow.Visibility = Visibility.Collapsed; });

            OAuthWindow.Visibility = Visibility.Visible;
            try
            {
                var result = await WebAuthenticator.AuthenticateAsync(new Uri(authorizeUri), new Uri(callbackUri), oauthCancellationSource.Token);
                MainWindow.BringToFront();
                OAuthWindow.Visibility = Visibility.Collapsed;
                Result.Text = $"Logged in. Info returned:";
                foreach(var value in result.Properties)
                    Result.Text += $"\n {value.Key} = {value.Value}";
            }
            catch (TaskCanceledException) {
                Result.Text = "Sign in cancelled";
            }
        }

        private void OAuthCancel_Click(object sender, RoutedEventArgs e)
        {
            oauthCancellationSource?.Cancel();            
        }
    }
}
