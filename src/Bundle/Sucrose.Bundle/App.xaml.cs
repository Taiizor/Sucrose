using System.Windows;
using System.Windows.Threading;
using SBM = Sucrose.Bundle.Main;
using SEAT = Skylark.Enum.AssemblyType;
using SHC = Skylark.Helper.Culture;
using SHV = Skylark.Helper.Versionly;

namespace Sucrose.Bundle
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += CurrentDomain_UnhandledException;
            Current.DispatcherUnhandledException += Current_DispatcherUnhandledException;
        }

        protected void Close(int Code = 0)
        {
            Environment.Exit(Code);
            Current.Shutdown(Code);
            Shutdown(Code);
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            string Lang = SHC.CurrentUITwoLetterISOLanguageName.ToUpperInvariant();

            if (!CheckLanguage(Lang))
            {
                Lang = "EN";
            }

            ResourceDictionary Resource = new()
            {
                Source = new Uri($"/Sucrose.Bundle;component/Properties/Resources_{Lang}.xaml", UriKind.Relative)
            };

            RemoveResource();

            Current.Resources.MergedDictionaries.Add(Resource);

            bool Silent = false;

            foreach (string Arg in e.Args)
            {
                if (Arg is "-s" or "/s" or "-silent" or "/silent")
                {
                    Silent = true;
                    break;
                }
            }

            base.OnStartup(e);

            SBM Main = new(Silent);

            if (Silent)
            {
                Main.Visibility = Visibility.Collapsed;
                Main.ShowInTaskbar = false;
                Main.Opacity = 0;
                Main.Hide();
            }

            Main.ShowDialog();

            Close(0);
        }

        private static bool CheckLanguage(string Lang)
        {
            try
            {
                return LoadComponent(new Uri($"/Sucrose.Bundle;component/Properties/Resources_{Lang}.xaml", UriKind.Relative)) is ResourceDictionary;
            }
            catch
            {
                return false;
            }
        }

        private static void RemoveResource()
        {
            List<ResourceDictionary> Resources = Current.Resources.MergedDictionaries
                .Where(Resource => !string.IsNullOrEmpty(Resource.Source?.ToString()) && Resource.Source.ToString().Contains("/Properties/Resources_"))
                .ToList();

            foreach (ResourceDictionary Resource in Resources)
            {
                Current.Resources.MergedDictionaries.Remove(Resource);
            }
        }

        private void CurrentDomain_UnhandledException(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = e.ExceptionObject as Exception;

            if (Exception != null)
            {
                MessageBox.Show(Exception.Message + Environment.NewLine + Environment.NewLine + Exception.StackTrace, $"Bundle Error Information - v{SHV.Auto(SEAT.Entry)}", MessageBoxButton.OK, MessageBoxImage.Error);

                Close(1);
            }
        }

        private void Current_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            if (Exception != null)
            {
                MessageBox.Show(Exception.Message + Environment.NewLine + Environment.NewLine + Exception.StackTrace, $"Bundle Error Information - v{SHV.Auto(SEAT.Entry)}", MessageBoxButton.OK, MessageBoxImage.Error);

                e.Handled = true;

                Close(1);
            }
        }
    }
}