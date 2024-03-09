using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Oracle.ManagedDataAccess.Client;
using Rhythm.Activation;
using Rhythm.Contracts.Services;
using Rhythm.Views;

namespace Rhythm.Services;

public class ActivationService : IActivationService
{
    private readonly ActivationHandler<LaunchActivatedEventArgs> _defaultHandler;
    private readonly IEnumerable<IActivationHandler> _activationHandlers;
    private readonly IThemeSelectorService _themeSelectorService;
    private UIElement? _shell = null;

    public ActivationService(ActivationHandler<LaunchActivatedEventArgs> defaultHandler, IEnumerable<IActivationHandler> activationHandlers, IThemeSelectorService themeSelectorService)
    {
        _defaultHandler = defaultHandler;
        _activationHandlers = activationHandlers;
        _themeSelectorService = themeSelectorService;
    }

    public async Task ActivateAsync(object activationArgs)
    {
        // Execute tasks before activation.
        await InitializeAsync();

        // Connect to database
        App.GetService<IDatabaseService>().ConnectToOracle();

        // Set the MainWindow Content.
        if (App.MainWindow.Content == null)
        {
            Windows.Storage.ApplicationDataContainer localSettings = Windows.Storage.ApplicationData.Current.LocalSettings;
            if (localSettings.Values["IsAuthenticated"] != null && bool.Parse(localSettings.Values["IsAuthenticated"].ToString() ?? "false") && localSettings.Values["UserId"] != null)
            {
                var userId = localSettings.Values["UserId"].ToString().Replace("\"", "");
                var connection = App.GetService<IDatabaseService>().GetOracleConnection();
                var command = connection.CreateCommand();
                command.CommandText = "SELECT * FROM users WHERE user_id = :userId";
                command.Parameters.Add(new OracleParameter("userId", userId));
                var reader = await command.ExecuteReaderAsync();
                if (reader.Read())
                {
                    _shell = App.GetService<ShellPage>();
                }
                else
                {
                    await App.GetService<ILocalSettingsService>().ClearAll();
                    _shell = App.GetService<LoginPage>();
                }
                _shell = App.GetService<ShellPage>();
            }
            else
            {
                _shell = App.GetService<LoginPage>();
            }
            App.MainWindow.Content = _shell ?? new Frame();
        }

        // Handle activation via ActivationHandlers.
        await HandleActivationAsync(activationArgs);

        // Activate the MainWindow.
        App.MainWindow.Activate();

        // Execute tasks after activation.
        await StartupAsync();
    }

    private async Task HandleActivationAsync(object activationArgs)
    {
        var activationHandler = _activationHandlers.FirstOrDefault(h => h.CanHandle(activationArgs));

        if (activationHandler != null)
        {
            await activationHandler.HandleAsync(activationArgs);
        }

        if (_defaultHandler.CanHandle(activationArgs))
        {
            await _defaultHandler.HandleAsync(activationArgs);
        }
    }

    private async Task InitializeAsync()
    {
        await _themeSelectorService.InitializeAsync().ConfigureAwait(false);
        await Task.CompletedTask;
    }

    private async Task StartupAsync()
    {
        await _themeSelectorService.SetRequestedThemeAsync();
        await Task.CompletedTask;
    }
}
