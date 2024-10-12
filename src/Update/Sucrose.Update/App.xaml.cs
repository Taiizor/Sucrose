using System.Globalization;
using System.Windows;
using SHC = Skylark.Helper.Culture;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMMRM = Sucrose.Memory.Manage.Readonly.Mutex;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;
using SSCHU = Sucrose.Shared.Core.Helper.Update;
using SSSHE = Sucrose.Shared.Space.Helper.Extension;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SUMI = Sucrose.Update.Manage.Internal;
using SUVMW = Sucrose.Update.View.MainWindow;
using SMMU = Sucrose.Manager.Manage.Update;
using SMMCU = Sucrose.Memory.Manage.Constant.Update;

namespace Sucrose.Update
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static bool HasError { get; set; } = true;

        public App()
        {
            AppDomain.CurrentDomain.FirstChanceException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_FirstChanceException(Exception);

                //Close();
                //Message(Exception);
            };

            AppDomain.CurrentDomain.UnhandledException += async (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                await SSWW.Watch_GlobalUnhandledException(Exception);

                //Close();
                Message(Exception);
            };

            TaskScheduler.UnobservedTaskException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception);
            };

            Current.DispatcherUnhandledException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception);
            };

            SHC.All = new CultureInfo(SMMM.Culture, true);
        }

        protected void Close()
        {
            SMMI.UpdateSettingManager.SetSetting(SMMCU.UpdateState, false);

            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(Exception Exception)
        {
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.UpdateLogManager.LogFile();
                string Text = SRER.GetValue("Update", "HelpText");
                string Source = SSSHE.Change(SUMI.Source, SSCHU.GetDescription(SSCEUET.Executable));

                SSSHW.Start(SMMRA.Update, Exception, Path, Source, Text);

                Close();
            }
        }

        protected void Configure(bool Silent)
        {
            SUVMW MainWindow = new(Silent);

            if (Silent)
            {
                MainWindow.Visibility = Visibility.Collapsed;
                MainWindow.ShowInTaskbar = false;
                MainWindow.Opacity = 0;
                MainWindow.Hide();
            }

            MainWindow.ShowDialog();

            Close();
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            //

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMMRM.Update, SMMRA.Update))
            {
                SMMI.UpdateSettingManager.SetSetting(SMMCU.UpdateState, false);

                Configure(e.Args.Any());
            }
            else
            {
                Close();
            }
        }
    }
}