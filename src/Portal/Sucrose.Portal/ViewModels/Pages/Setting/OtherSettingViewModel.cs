using CommunityToolkit.Mvvm.ComponentModel;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Appearance;
using Wpf.Ui.Controls;
using Button = Wpf.Ui.Controls.Button;
using DialogResult = System.Windows.Forms.DialogResult;
using FolderBrowserDialog = System.Windows.Forms.FolderBrowserDialog;
using OpenFileDialog = Microsoft.Win32.OpenFileDialog;
using SEOST = Skylark.Enum.OperatingSystemType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGCLLC = Sucrose.Grpc.Common.Launcher.LauncherClient;
using SGCSLCS = Sucrose.Grpc.Client.Services.LauncherClientService;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDESCT = Sucrose.Shared.Dependency.Enum.SchedulerCommandsType;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSRHR = Sucrose.Shared.Resources.Helper.Resources;
using SSSHC = Sucrose.Shared.Space.Helper.Copy;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWUD = Skylark.Wing.Utility.Desktop;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class OtherSettingViewModel : ObservableObject, INavigationAware, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public OtherSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        private void InitializeViewModel()
        {
            TextBlock AppearanceBehavior = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Kanca"
            };

            Contents.Add(AppearanceBehavior);

            SPVCEC DiscordHook = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            DiscordHook.Title.Text = "Discord";
            DiscordHook.LeftIcon.Symbol = SymbolRegular.TetrisApp24;
            DiscordHook.Description.Text = "Sucrose, Discord ile arasındaki bağlantıyı sağlar.";

            ToggleSwitch DiscordState = new()
            {
                IsChecked = SPMM.DiscordState
            };

            DiscordState.Checked += (s, e) => DiscordStateChecked(true);
            DiscordState.Unchecked += (s, e) => DiscordStateChecked(false);

            DiscordHook.HeaderFrame = DiscordState;

            StackPanel DiscordContent = new();

            StackPanel DiscordRefreshContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock DiscordRefreshText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Bağlantı Yenileme:",
            };

            ToggleSwitch DiscordRefresh = new()
            {
                IsChecked = SPMM.DiscordRefresh
            };

            DiscordRefresh.Checked += (s, e) => DiscordRefreshChecked(true);
            DiscordRefresh.Unchecked += (s, e) => DiscordRefreshChecked(false);

            StackPanel DiscordDelayContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock DiscordDelayText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                Text = "Yenileme Süresi (Saniye):",
                FontWeight = FontWeights.SemiBold
            };

            NumberBox DiscordDelay = new()
            {
                ClearButtonEnabled = false,
                Value = SPMM.DiscordDelay,
                Maximum = 3600,
                Minimum = 60
            };

            DiscordDelay.ValueChanged += (s, e) => DiscordDelayChanged(DiscordDelay.Value);

            DiscordRefreshContent.Children.Add(DiscordRefreshText);
            DiscordRefreshContent.Children.Add(DiscordRefresh);

            DiscordDelayContent.Children.Add(DiscordDelayText);
            DiscordDelayContent.Children.Add(DiscordDelay);

            DiscordContent.Children.Add(DiscordRefreshContent);
            DiscordContent.Children.Add(DiscordDelayContent);

            DiscordHook.FooterCard = DiscordContent;

            Contents.Add(DiscordHook);

            _isInitialized = true;
        }

        public void OnNavigatedTo()
        {
            //
        }

        public void OnNavigatedFrom()
        {
            //Dispose();
        }

        private void DiscordStateChecked(bool State)
        {
            SMMI.HookSettingManager.SetSetting(SMC.DiscordState, State);
        }

        private void DiscordRefreshChecked(bool State)
        {
            SMMI.HookSettingManager.SetSetting(SMC.DiscordRefresh, State);
        }

        private void DiscordDelayChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SPMM.DiscordDelay)
            {
                SMMI.HookSettingManager.SetSetting(SMC.DiscordDelay, NewValue);
            }
        }

        public void Dispose()
        {
            Contents.Clear();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}