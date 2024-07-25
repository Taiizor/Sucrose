using Wpf.Ui.Abstractions;

namespace Sucrose.Portal.Providers
{
    /// <summary>
    /// Service that provides pages for navigation.
    /// </summary>
    public class NavigationViewPageProvider(IServiceProvider serviceProvider) : INavigationViewPageProvider
    {
        /// <inheritdoc />
        public object? GetPage(Type pageType)
        {
            return serviceProvider.GetService(pageType);
        }
    }
}