using System.IO;
using System.Reflection;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHC = Skylark.Helper.Culture;
using SHN = Skylark.Helper.Numeric;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSSHS = Sucrose.Shared.Space.Helper.Size;
using SSSSS = Skylark.Struct.Storage.StorageStruct;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SPVCDS = Sucrose.Portal.Views.Controls.Display.Screen;
using SSMMS = Skylark.Struct.Monitor.MonitorStruct;
using SWUS = Skylark.Wing.Utility.Screene;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMC = Sucrose.Memory.Constant;
using System.Threading;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DisplayType;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// DisplayPreferences.xaml etkileşim mantığı
    /// </summary>
    public partial class DisplayPreferences : ContentDialog, IDisposable
    {
        public DisplayPreferences() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private async Task ScreenMonitor()
        {
            int ScreenCount = SWUS.Screens.Count();

            while (SMMM.ScreenIndex > ScreenCount - 1)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.ScreenIndex, SMMM.ScreenIndex - 1);

                await Task.Delay(10);
            }

            for (int Count = 0; Count < ScreenCount; Count++)
            {
                SPVCDS Screen = new();

                if (SMMM.ScreenIndex == Count)
                {
                    Screen.Border.BorderBrush = Brushes.CornflowerBlue;
                }

                Screen.Index.Text = $"{Count + 1}";
                Screen.MouseLeftButtonDown += ScreenClicked;

                Contents.Children.Add(Screen);
            }

            Contents.InvalidateMeasure();

            await Task.CompletedTask;
        }

        private void ScreenClicked(object sender, MouseButtonEventArgs e)
        {
            SPVCDS ScreenMonitor = sender as SPVCDS;

            foreach (UIElement Child in Contents.Children)
            {
                if (Child is SPVCDS Screen)
                {
                    if (Screen == ScreenMonitor)
                    {
                        Screen.Border.BorderBrush = Brushes.CornflowerBlue;

                        SMMI.EngineSettingManager.SetSetting(SMC.ScreenIndex, Convert.ToInt32(Screen.Index.Text) - 1);
                    }
                    else
                    {
                        Screen.Border.BorderBrush = SSRER.GetResource<Brush>("ControlAltFillColorTertiaryBrush");
                    }
                }
            }
        }

        private async void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            switch (SPMM.DisplayType)
            {
                case SSDEDT.Expand:
                    //await ExpandMonitor();
                    break;
                case SSDEDT.Duplicate:
                    //await DuplicateMonitor();
                    break;
                default:
                    await ScreenMonitor();
                    break;
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}