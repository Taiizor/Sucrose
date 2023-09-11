using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using System.Globalization;
using System.Windows;
using Wpf.Ui;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMAC = Sucrose.Portal.Models.AppConfig;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPSAHS = Sucrose.Portal.Services.ApplicationHostService;
using SPSCIW = Sucrose.Portal.Services.Contracts.IWindow;
using SPSPS = Sucrose.Portal.Services.PageService;
using SPSWPS = Sucrose.Portal.Services.WindowsProviderService;
using SPVMPDSVM = Sucrose.Portal.ViewModels.Pages.DonateSettingViewModel;
using SPVMPGSVM = Sucrose.Portal.ViewModels.Pages.GeneralSettingViewModel;
using SPVMPLVM = Sucrose.Portal.ViewModels.Pages.LibraryViewModel;
using SPVMPOSVM = Sucrose.Portal.ViewModels.Pages.OtherSettingViewModel;
using SPVMPPESVM = Sucrose.Portal.ViewModels.Pages.PerformanceSettingViewModel;
using SPVMPPLSVM = Sucrose.Portal.ViewModels.Pages.PersonalSettingViewModel;
using SPVMPSSVM = Sucrose.Portal.ViewModels.Pages.SystemSettingViewModel;
using SPVMPSVM = Sucrose.Portal.ViewModels.Pages.StoreViewModel;
using SPVMPWSVM = Sucrose.Portal.ViewModels.Pages.WallpaperSettingViewModel;
using SPVMWMWVM = Sucrose.Portal.ViewModels.Windows.MainWindowViewModel;
using SPVPLP = Sucrose.Portal.Views.Pages.LibraryPage;
using SPVPSDSP = Sucrose.Portal.Views.Pages.Setting.DonateSettingPage;
using SPVPSGSP = Sucrose.Portal.Views.Pages.Setting.GeneralSettingPage;
using SPVPSOSP = Sucrose.Portal.Views.Pages.Setting.OtherSettingPage;
using SPVPSP = Sucrose.Portal.Views.Pages.StorePage;
using SPVPSPESP = Sucrose.Portal.Views.Pages.Setting.PerformanceSettingPage;
using SPVPSPLSP = Sucrose.Portal.Views.Pages.Setting.PersonalSettingPage;
using SPVPSSSP = Sucrose.Portal.Views.Pages.Setting.SystemSettingPage;
using SPVPSWSP = Sucrose.Portal.Views.Pages.Setting.WallpaperSettingPage;
using SPVWMW = Sucrose.Portal.Views.Windows.MainWindow;
using SSRHR = Sucrose.Shared.Resources.Helper.Resources;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSWDEMB = Sucrose.Shared.Watchdog.DarkErrorMessageBox;
using SSWLEMB = Sucrose.Shared.Watchdog.LightErrorMessageBox;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Portal
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static bool HasError { get; set; } = true;

        // The .NET Generic Host provides dependency injection, configuration, logging, and other services.
        // https://docs.microsoft.com/dotnet/core/extensions/generic-host
        // https://docs.microsoft.com/dotnet/core/extensions/dependency-injection
        // https://docs.microsoft.com/dotnet/core/extensions/configuration
        // https://docs.microsoft.com/dotnet/core/extensions/logging
        private static readonly IHost _host = Host
            .CreateDefaultBuilder()
            .ConfigureAppConfiguration(configure =>
                {
                    configure.SetBasePath(AppContext.BaseDirectory);
                }
            )
            .ConfigureServices((context, services) =>
                {
                    // App Host
                    services.AddHostedService<SPSAHS>();

                    // Page resolver service
                    services.AddSingleton<IPageService, SPSPS>();

                    // Theme manipulation
                    services.AddSingleton<IThemeService, ThemeService>();

                    // TaskBar manipulation
                    services.AddSingleton<ITaskBarService, TaskBarService>();

                    // 
                    services.AddSingleton<ISnackbarService, SnackbarService>();

                    // Service containing navigation, same as INavigationWindow... but without window
                    services.AddSingleton<INavigationService, NavigationService>();

                    // 
                    services.AddSingleton<IContentDialogService, ContentDialogService>();

                    // Main window with navigation
                    services.AddSingleton<SPSCIW, SPVWMW>();
                    services.AddSingleton<SPVMWMWVM>();
                    services.AddSingleton<SPSWPS>();

                    // Views and ViewModels
                    services.AddTransient<SPVPLP>();
                    services.AddTransient<SPVMPLVM>();

                    services.AddTransient<SPVPSP>();
                    services.AddTransient<SPVMPSVM>();

                    services.AddTransient<SPVPSDSP>();
                    services.AddTransient<SPVMPDSVM>();

                    services.AddTransient<SPVPSGSP>();
                    services.AddTransient<SPVMPGSVM>();

                    services.AddTransient<SPVPSOSP>();
                    services.AddTransient<SPVMPOSVM>();

                    services.AddTransient<SPVPSPESP>();
                    services.AddTransient<SPVMPPESVM>();

                    services.AddTransient<SPVPSPLSP>();
                    services.AddTransient<SPVMPPLSVM>();

                    services.AddTransient<SPVPSSSP>();
                    services.AddTransient<SPVMPSSVM>();

                    services.AddTransient<SPVPSWSP>();
                    services.AddTransient<SPVMPWSVM>();

                    // Configuration
                    services.Configure<SPMAC>(context.Configuration.GetSection(nameof(SPMAC)));
                }
            )
            .Build();

        public App()
        {
            AppDomain.CurrentDomain.FirstChanceException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_FirstChanceException(Exception);

                //Close();
                //Message(Exception.Message);
            };

            AppDomain.CurrentDomain.UnhandledException += (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                SSWW.Watch_GlobalUnhandledException(Exception);

                //Close();
                Message(Exception.Message);
            };

            TaskScheduler.UnobservedTaskException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception.Message);
            };

            Current.DispatcherUnhandledException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception.Message);
            };

            SHC.All = new CultureInfo(SMMM.Culture, true);
        }

        /// <summary>
        /// Gets registered service.
        /// </summary>
        /// <typeparam name="T">Type of the service to get.</typeparam>
        /// <returns>Instance of the service or <see langword="null"/>.</returns>
        public static T GetService<T>() where T : class
        {
            return _host.Services.GetService(typeof(T)) as T ?? null;
        }

        protected void Close()
        {
            _host.StopAsync().Wait();

            _host.Dispose();

            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(string Message)
        {
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.PortalLogManager.LogFile();

                switch (SPMM.Theme)
                {
                    case SEWTT.Dark:
                        SSWDEMB DarkMessageBox = new(Message, Path);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SSWLEMB LightMessageBox = new(Message, Path);
                        LightMessageBox.ShowDialog();
                        break;
                }

                Close();
            }
        }

        protected void Configure()
        {
            _host.Start();

            //Main Interface = new();
            //Interface.Show();
        }

        /// <summary>
        /// Occurs when the application is closing.
        /// </summary>
        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            Close();
        }

        /// <summary>
        /// Occurs when the application is loading.
        /// </summary>
        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SSRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnLastWindowClose;

            if (SPMM.Mutex.WaitOne(TimeSpan.Zero, true) && SSSHP.WorkCount(SMR.Portal) <= 1)
            {
                SPMM.Mutex.ReleaseMutex();

                Configure();
            }
            else
            {
                Close();
            }
        }
    }
}