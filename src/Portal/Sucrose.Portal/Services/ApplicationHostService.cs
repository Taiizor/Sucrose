using Microsoft.Extensions.Hosting;
using System.Windows;
using SPSCIW = Sucrose.Portal.Services.Contracts.IWindow;
using SPVWMW = Sucrose.Portal.Views.Windows.MainWindow;

namespace Sucrose.Portal.Services
{
    /// <summary>
    /// Managed host of the application.
    /// </summary>
    public class ApplicationHostService(IServiceProvider ServiceProvider) : IHostedService
    {
        private readonly IServiceProvider _ServiceProvider = ServiceProvider;

        /// <summary>
        /// Triggered when the application host is ready to start the service.
        /// </summary>
        /// <param name="CancellationToken">Indicates that the start process has been aborted.</param>
        public async Task StartAsync(CancellationToken CancellationToken)
        {
            await HandleActivationAsync();
        }

        /// <summary>
        /// Triggered when the application host is performing a graceful shutdown.
        /// </summary>
        /// <param name="CancellationToken">Indicates that the shutdown process should no longer be graceful.</param>
        public async Task StopAsync(CancellationToken CancellationToken)
        {
            await Task.CompletedTask;
        }

        /// <summary>
        /// Creates main window during activation.
        /// </summary>
        private async Task HandleActivationAsync()
        {
            await Task.CompletedTask;

            if (!Application.Current.Windows.OfType<SPVWMW>().Any())
            {
                SPSCIW MainWindow = _ServiceProvider.GetService(typeof(SPSCIW)) as SPSCIW;
                MainWindow?.Show();
            }

            await Task.CompletedTask;
        }
    }
}