using Microsoft.Extensions.DependencyInjection;
using System.Windows;

namespace Sucrose.Portal.Services
{
    public class WindowsProviderService(IServiceProvider ServiceProvider)
    {
        private readonly IServiceProvider _ServiceProvider = ServiceProvider;

        public void Show<T>() where T : class
        {
            if (!typeof(Window).IsAssignableFrom(typeof(T)))
            {
                throw new InvalidOperationException($"The window class should be derived from {typeof(Window)}.");
            }

            if (_ServiceProvider.GetService<T>() is not Window WindowInstance)
            {
                throw new InvalidOperationException("Window is not registered as service.");
            }

            WindowInstance.Owner = Application.Current.MainWindow;
            WindowInstance.Show();
        }
    }
}