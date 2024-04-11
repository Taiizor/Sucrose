using System.Windows.Input;

namespace Sucrose.Shared.Engine.CefSharp.Extension
{
    internal class RelayCommand(Action<object> commandHandler, Func<object, bool> canExecuteHandler = null) : ICommand
    {
        public event EventHandler CanExecuteChanged;

        public RelayCommand(Action commandHandler, Func<bool> canExecuteHandler = null) : this(_ => commandHandler(), canExecuteHandler == null ? null : new Func<object, bool>(_ => canExecuteHandler()))
        {
        }

        public void Execute(object parameter)
        {
            commandHandler(parameter);
        }

        public bool CanExecute(object parameter)
        {
            return canExecuteHandler == null || canExecuteHandler(parameter);
        }

        public void RaiseCanExecuteChanged()
        {
            CanExecuteChanged?.Invoke(this, EventArgs.Empty);
        }
    }

    internal class RelayCommand<T>(Action<T> commandHandler, Func<T, bool> canExecuteHandler = null) : RelayCommand(o => commandHandler(o is T t ? t : default), canExecuteHandler == null ? null : new Func<object, bool>(o => canExecuteHandler(o is T t ? t : default)))
    {
    }
}