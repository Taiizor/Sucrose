using System.Windows;
using Wpf.Ui;

namespace Sucrose.Portal.Services
{
    /// <summary>
    /// Service that provides pages for navigation.
    /// </summary>
    /// <remarks>
    /// Creates new instance and attaches the <see cref="IServiceProvider"/>.
    /// </remarks>
    public class PageService(IServiceProvider ServiceProvider) : IPageService
    {
        /// <summary>
        /// Service which provides the instances of pages.
        /// </summary>
        private readonly IServiceProvider _ServiceProvider = ServiceProvider;

        /// <inheritdoc />
        public T GetPage<T>() where T : class
        {
            if (!typeof(FrameworkElement).IsAssignableFrom(typeof(T)))
            {
                throw new InvalidOperationException("The page should be a WPF control.");
            }

            return (T)_ServiceProvider.GetService(typeof(T));
        }

        /// <inheritdoc />
        public FrameworkElement GetPage(Type PageType)
        {
            if (!typeof(FrameworkElement).IsAssignableFrom(PageType))
            {
                throw new InvalidOperationException("The page should be a WPF control.");
            }

            return _ServiceProvider.GetService(PageType) as FrameworkElement;
        }
    }
}