using System.Windows;

namespace Sucrose.Portal.Services.Contracts
{
    public interface IWindow
    {
        event RoutedEventHandler Loaded;

        void Show();
    }
}