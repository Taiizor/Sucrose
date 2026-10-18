using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Controls;
using SMMC = Sucrose.Manager.Manage.Cycling;
using SMMCC = Sucrose.Memory.Manage.Constant.Cycling;
using SMMI = Sucrose.Manager.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDETCT = Sucrose.Shared.Dependency.Enum.TransitionCycleType;
using SSDMMC = Sucrose.Shared.Dependency.Manage.Manager.Cycling;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// WallpaperCycling.xaml etkileşim mantığı
    /// </summary>
    public partial class WallpaperCycling : ContentDialog, IDisposable
    {
        public WallpaperCycling(ContentDialogHost? contentDialogHost) : base(contentDialogHost)
        {
            InitializeComponent();
        }

        private void ResetList_Click(object sender, RoutedEventArgs e)
        {
            SMMI.CyclingSettingManager.SetSetting(SMMCC.Exclusion, new List<string>());
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            CyclingState.IsChecked = SMMC.Active;

            CyclingState.Checked += (s, e) => CyclingStateChecked(true);
            CyclingState.Unchecked += (s, e) => CyclingStateChecked(false);

            TimeDuration.Value = SMMC.TransitionTime;

            TimeDuration.ValueChanged += (s, e) => TimeDurationChanged(TimeDuration.Value);

            TransitionType.SelectionChanged += (s, e) => TransitionTypeSelected(TransitionType.SelectedIndex);

            foreach (SSDETCT Type in Enum.GetValues(typeof(SSDETCT)))
            {
                TransitionType.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "TransitionCycleType", $"{Type}")
                });
            }

            TransitionType.SelectedIndex = (int)SSDMMC.TransitionType;
        }

        private void CyclingStateChecked(bool State)
        {
            SMMI.CyclingSettingManager.SetSetting(SMMCC.Active, State);
        }

        private void TransitionTypeSelected(int Index)
        {
            SSDETCT NewType = (SSDETCT)Index;

            if (NewType != SSDMMC.TransitionType)
            {
                SMMI.CyclingSettingManager.SetSetting(SMMCC.TransitionType, NewType);
            }
        }

        private void TimeDurationChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMC.TransitionTime)
            {
                SMMI.CyclingSettingManager.SetSetting(SMMCC.TransitionTime, NewValue);
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}