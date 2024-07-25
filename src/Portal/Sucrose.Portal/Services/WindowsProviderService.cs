using Microsoft.Extensions.DependencyInjection;
using System.Windows;

namespace Sucrose.Portal.Services
{
    public class WindowsProviderService(IServiceProvider serviceProvider)
    {
        public void Show<T>() where T : class
        {
            if (!typeof(Window).IsAssignableFrom(typeof(T)))
            {
                throw new InvalidOperationException($"The window class should be derived from {typeof(Window)}.");
            }

            Window windowInstance = serviceProvider.GetService<T>() as Window ?? throw new InvalidOperationException("Window is not registered as service.");

            windowInstance.Owner = Application.Current.MainWindow;
            windowInstance.Show();
        }
    }
}