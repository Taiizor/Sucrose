using Microsoft.UI.Xaml.Controls;
using Oracle.ManagedDataAccess.Client;
using Rhythm.Contracts.Services;
using Rhythm.ViewModels;

namespace Rhythm.Views;

public sealed partial class MainPage : Page
{
    public static readonly string PageName = "Home";

    public static readonly bool IsPageHidden = false;

    public MainViewModel ViewModel
    {
        get;
    }

    public MainPage()
    {
        ViewModel = App.GetService<MainViewModel>();
        InitializeComponent();
    }

    private void Button_ClickAsync(object sender, Microsoft.UI.Xaml.RoutedEventArgs e)
    {
        var stopwatch = new System.Diagnostics.Stopwatch();
        stopwatch.Start();
        var connection = App.GetService<IDatabaseService>().GetOracleConnection();
        var command = new OracleCommand("SELECT 1 FROM DUAL", connection);
        command.ExecuteNonQuery();
        stopwatch.Stop();
        App.MainWindow.ShowMessageDialogAsync($"Elapsed Time: {stopwatch.ElapsedMilliseconds}ms", "Database Connection Test");
    }
}
