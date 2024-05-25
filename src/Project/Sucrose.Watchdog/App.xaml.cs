using System.Globalization;
using System.Windows;
using Application = System.Windows.Application;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSHWE = Sucrose.Shared.Space.Helper.WatchException;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWVDEMB = Sucrose.Watchdog.View.DarkErrorMessageBox;
using SWVLEMB = Sucrose.Watchdog.View.LightErrorMessageBox;

namespace Sucrose.Watchdog
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
                //Message(Exception.Message);
            };

            AppDomain.CurrentDomain.UnhandledException += async (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                await SSWW.Watch_GlobalUnhandledException(Exception);

                //Close();
                Message(Exception.Message);
            };

            TaskScheduler.UnobservedTaskException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception.Message);
            };

            Current.DispatcherUnhandledException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception.Message);
            };

            SHC.All = new CultureInfo(SMMM.Culture, true);
        }

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(string Message)
        {
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.WatchdogLogManager.LogFile();

                switch (SSDMM.ThemeType)
                {
                    case SEWTT.Dark:
                        SWVDEMB DarkMessageBox = new(Message, Path);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SWVLEMB LightMessageBox = new(Message, Path);
                        LightMessageBox.ShowDialog();
                        break;
                }

                Close();
            }
        }

        protected void Configure(string[] Args)
        {
            if (Args.Any())
            {
                string Decode = SSECCE.BaseToText(Args.First());
                string[] Arguments = Decode.Split(SMR.ValueSeparatorChar);

                if (Arguments.Any() && (Arguments.Count() == 3 || Arguments.Count() == 5))
                {
                    string Path = Arguments[2];
                    string Application = Arguments[0];
                    string Message = SSSHWE.Convert(Arguments[1]).Message;
                    string Text = Arguments.Count() == 5 ? Arguments[4] : string.Empty;
                    string Source = Arguments.Count() == 5 ? Arguments[3] : string.Empty;

                    SSSHP.Kill(Application);

                    switch (SSDMM.ThemeType)
                    {
                        case SEWTT.Dark:
                            SWVDEMB DarkMessageBox = new(Message, Path, Source, Text);
                            DarkMessageBox.ShowDialog();
                            break;
                        default:
                            SWVLEMB LightMessageBox = new(Message, Path, Source, Text);
                            LightMessageBox.ShowDialog();
                            break;
                    }

                    Close();
                }
                else
                {
                    Close();
                }
            }
            else
            {
                Close();
            }
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnLastWindowClose;

            SSSHU.ControlGuid();

            Configure(e.Args);
        }
    }
}