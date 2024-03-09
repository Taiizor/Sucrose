using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;
using Rhythm.Contracts.Services;
using Rhythm.Helpers;
using Rhythm.ViewModels;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;
using Windows.Storage.Pickers;

namespace Rhythm.Views;


// TODO: Set the URL for your privacy policy by updating SettingsPage_PrivacyTermsLink.NavigateUri in Resources.resw.
public sealed partial class SettingsPage : Page
{
    public static readonly string PageName = "Settings";

    public static readonly bool IsPageHidden = false;

    public SettingsViewModel ViewModel
    {
        get;
    }

    public SettingsPage()
    {
        ViewModel = App.GetService<SettingsViewModel>();
        InitializeComponent();
    }

    private void ComboBox_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        var selectedText = ((ComboBoxItem)e.AddedItems[0]).Content.ToString();
        var theme = selectedText switch
        {

            "Light" => ElementTheme.Light,
            "Dark" => ElementTheme.Dark,
            _ => ElementTheme.Default
        };
        ViewModel.SwitchThemeCommand.Execute(theme);
    }


    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        var theme = ViewModel.currentTheme;
        switch (theme)
        {
            case "Light":
                ThemeComboBox.SelectedIndex = 0;
                break;
            case "Dark":
                ThemeComboBox.SelectedIndex = 1;
                break;
            default:
                ThemeComboBox.SelectedIndex = 2;
                break;
        }
        SaveUsernameButton.IsEnabled = false;
        SavePasswordButton.IsEnabled = false;
    }

    private void LogoutButton_Click(object sender, RoutedEventArgs e)
    {
        App.GetService<ILocalSettingsService>().ClearAll();
        App.MainWindow.Content = App.GetService<LoginPage>();
    }

    private async void PickAPhotoButton_Click(object sender, RoutedEventArgs e)
    {
        var openPicker = new FileOpenPicker();
        var window = App.MainWindow;
        var hWnd = WinRT.Interop.WindowNative.GetWindowHandle(window);
        WinRT.Interop.InitializeWithWindow.Initialize(openPicker, hWnd);
        openPicker.ViewMode = PickerViewMode.Thumbnail;
        openPicker.SuggestedStartLocation = PickerLocationId.PicturesLibrary;
        openPicker.FileTypeFilter.Add(".jpg");
        openPicker.FileTypeFilter.Add(".jpeg");
        openPicker.FileTypeFilter.Add(".png");
        var file = await openPicker.PickSingleFileAsync();
        if (file != null)
        {
            var buffer = await FileIO.ReadBufferAsync(file);
            var dataReader = Windows.Storage.Streams.DataReader.FromBuffer(buffer);
            var bytes = new byte[buffer.Length];
            dataReader.ReadBytes(bytes);
            UserImage.Source = await BitmapHelper.GetBitmapAsync(bytes);
            if (ViewModel.currentUser is not null)
            {

                ViewModel.currentUser.UserImage = bytes;
                await ViewModel.UpdateUserImage(bytes);
            }
        }
    }

    public static string relativize(DateTime date)
    {
        var span = DateTime.Now - date;
        if (span.Days > 365)
        {
            var years = span.Days / 365;
            if (span.Days % 365 != 0)
            {
                years += 1;
            }
            return $"about {years} {(years == 1 ? "year" : "years")} ago";
        }
        if (span.Days > 30)
        {
            var months = span.Days / 30;
            if (span.Days % 31 != 0)
            {
                months += 1;
            }
            return $"about {months} {(months == 1 ? "month" : "months")} ago";
        }
        if (span.Days > 0)
        {
            return $"about {span.Days} {(span.Days == 1 ? "day" : "days")} ago";
        }
        if (span.Hours > 0)
        {
            return $"about {span.Hours} {(span.Hours == 1 ? "hour" : "hours")} ago";
        }
        if (span.Minutes > 0)
        {
            return $"about {span.Minutes} {(span.Minutes == 1 ? "minute" : "minutes")} ago";
        }
        if (span.Seconds > 5)
        {
            return $"about {span.Seconds} seconds ago";
        }
        return "just now";
    }

    private async Task LoadUserData()
    {
        await ViewModel.LoadUserAsync();
        if (ViewModel.currentUser is not null)
        {
            Username.Text = ViewModel.currentUser.UserName;
            CreatedAt.Text = "joined " + relativize(ViewModel.currentUser.CreatedAt);
            UsernameTextBox.Text = ViewModel.currentUser.UserName;
            ViewModel.UserLoaded = true;
            await ViewModel.LoadUserImage();
            if (ViewModel.currentUser.UserImage.Length > 0)
            {
                var bitmap = await BitmapHelper.GetBitmapAsync(ViewModel.currentUser.UserImage);
                UserImage.Source = bitmap;
            }
        }
    }

    protected async override void OnNavigatedTo(NavigationEventArgs e)
    {
        base.OnNavigatedTo(e);
        await LoadUserData();
    }

    private void CopyButton_Click(object sender, RoutedEventArgs e)
    {
        if (ViewModel.currentUser is not null)
        {
            var dataPackage = new DataPackage();
            dataPackage.SetText(ViewModel.currentUser.UserId);
            Clipboard.SetContent(dataPackage);
            App.GetService<IAppNotificationService>().Show("UserIdCopiedNotification".GetLocalized());
        }
        else
        {
            App.GetService<IAppNotificationService>().Show("User ID not found");
        }
    }

    private async void SaveUsernameButton_Click(object sender, RoutedEventArgs e)
    {
        if (ViewModel.currentUser is not null)
        {
            var name = UsernameTextBox.Text;
            await ViewModel.UpdateUserName(name);
            Username.Text = ViewModel.currentUser.UserName;
            await App.MainWindow.ShowMessageDialogAsync("Username updated successfully");
        }
    }

    private async void SavePasswordButton_Click(object sender, RoutedEventArgs e)
    {
        if (ViewModel.currentUser is not null)
        {
            var password = NewPasswordBox.Password;
            await ViewModel.UpdateUserPassword(password);
            await App.MainWindow.ShowMessageDialogAsync("Password updated successfully");

        }
    }

    private void UsernameTextChanged(object sender, TextChangedEventArgs e)
    {
        if (UsernameStatus.Text.Contains("invalid"))
        {
            UsernameStatus.Visibility = Visibility.Visible;
            SaveUsernameButton.IsEnabled = false;
        }
    }

    private void PasswordContentChanged(object sender, RoutedEventArgs e)
    {
        if (PasswordStatus.Text.Contains("invalid"))
        {
            PasswordStatus.Visibility = Visibility.Visible;
            SavePasswordButton.IsEnabled = false;
        }
    }

    private async void DeleteAccountButton_Click(object sender, RoutedEventArgs e)
    {
        ContentDialog deleteAccountDialog = new ContentDialog
        {
            XamlRoot = this.XamlRoot,
            Title = "Delete Account",
            Content = "Are you sure you want to delete your account? This action cannot be undone.",
            PrimaryButtonText = "Delete",
            CloseButtonText = "Cancel"
        };
        ContentDialogResult result = await deleteAccountDialog.ShowAsync();
        if (result == ContentDialogResult.Primary && ViewModel.currentUser is not null)
        {
            await ViewModel.DeleteAccount();
            await App.GetService<ILocalSettingsService>().ClearAll();
            App.MainWindow.Content = App.GetService<LoginPage>();
        }
    }
}
