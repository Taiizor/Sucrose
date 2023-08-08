using Microsoft.Extensions.DependencyInjection;
using System.Windows;

namespace Sucrose.Portal.Services
{
    public class WindowsProviderService(IServiceProvider serviceProvider)
    {
        private readonly IServiceProvider _serviceProvider = serviceProvider;

        public void Show<T>() where T : class
        {
            if (!typeof(Window).IsAssignableFrom(typeof(T)))
            {
                throw new InvalidOperationException($"The window class should be derived from {typeof(Window)}.");
            }

            if (_serviceProvider.GetService<T>() is not Window windowInstance)
            {
                throw new InvalidOperationException("Window is not registered as service.");
            }

            windowInstance.Owner = Application.Current.MainWindow;
            windowInstance.Show();
        }
    }
}