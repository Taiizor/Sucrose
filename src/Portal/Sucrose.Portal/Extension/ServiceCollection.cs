using Microsoft.Extensions.DependencyInjection;
using Wpf.Ui.Abstractions;
using SPPNVPP = Sucrose.Portal.Providers.NavigationViewPageProvider;

namespace Sucrose.Portal.Extension
{
    /// <summary>
    /// Provides extension methods for <see cref="IServiceCollection"/> to support WPF UI navigation and services.
    /// </summary>
    public static class ServiceCollection
    {
        /// <summary>
        /// Adds the services necessary for page navigation within a WPF UI NavigationView.
        /// </summary>
        /// <param name="services">The <see cref="IServiceCollection"/> to add the services to.</param>
        /// <returns>The <see cref="IServiceCollection"/> so that additional calls can be chained.</returns>

        public static IServiceCollection AddNavigationViewPageProvider(this IServiceCollection services)
        {
            _ = services.AddSingleton<INavigationViewPageProvider, SPPNVPP>();

            return services;
        }
    }
}