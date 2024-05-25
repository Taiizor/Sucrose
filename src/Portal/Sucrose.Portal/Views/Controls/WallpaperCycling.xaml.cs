using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDETT = Sucrose.Shared.Dependency.Enum.TransitionType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// WallpaperCycling.xaml etkileşim mantığı
    /// </summary>
    public partial class WallpaperCycling : ContentDialog, IDisposable
    {
        public WallpaperCycling() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void ResetList_Click(object sender, RoutedEventArgs e)
        {
            SMMI.CyclingSettingManager.SetSetting(SMC.DisableCycyling, new List<string>());
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            Cycling.TitleText = SRER.GetValue("Portal", "WallpaperCycling", "Cycling");
            Cycling.DescriptionText = SRER.GetValue("Portal", "WallpaperCycling", "Cycling", "Description");

            ToggleSwitch CyclingState = new()
            {
                IsChecked = SMMM.Cycyling
            };

            CyclingState.Checked += (s, e) => CyclingStateChecked(true);
            CyclingState.Unchecked += (s, e) => CyclingStateChecked(false);

            Cycling.HeaderFrame = CyclingState;

            Time.TitleText = SRER.GetValue("Portal", "WallpaperCycling", "Time");
            Time.DescriptionText = SRER.GetValue("Portal", "WallpaperCycling", "Time", "Description");

            NumberBox TimeDuration = new()
            {
                Icon = new SymbolIcon(SymbolRegular.Timer24),
                IconPlacement = ElementPlacement.Left,
                ClearButtonEnabled = false,
                Value = SMMM.CycylingTime,
                MaxDecimalPlaces = 0,
                MaxLength = 3,
                Maximum = 999,
                Minimum = 1
            };

            TimeDuration.ValueChanged += (s, e) => TimeDurationChanged(TimeDuration.Value);

            Time.HeaderFrame = TimeDuration;

            Transition.TitleText = SRER.GetValue("Portal", "WallpaperCycling", "Transition");
            Transition.DescriptionText = SRER.GetValue("Portal", "WallpaperCycling", "Transition", "Description");

            ComboBox TransitionType = new();

            TransitionType.SelectionChanged += (s, e) => TransitionTypeSelected(TransitionType.SelectedIndex);

            foreach (SSDETT Type in Enum.GetValues(typeof(SSDETT)))
            {
                TransitionType.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "TransitionType", $"{Type}")
                });
            }

            TransitionType.SelectedIndex = (int)SSDMM.TransitionType;

            Transition.HeaderFrame = TransitionType;

            List.TitleText = SRER.GetValue("Portal", "WallpaperCycling", "List");
            List.DescriptionText = SRER.GetValue("Portal", "WallpaperCycling", "List", "Description");
        }

        private void CyclingStateChecked(bool State)
        {
            SMMI.CyclingSettingManager.SetSetting(SMC.Cycyling, State);
        }

        private void TransitionTypeSelected(int Index)
        {
            SSDETT NewStore = (SSDETT)Index;

            if (NewStore != SSDMM.TransitionType)
            {
                SMMI.CyclingSettingManager.SetSetting(SMC.TransitionType, NewStore);
            }
        }

        private void TimeDurationChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.CycylingTime)
            {
                SMMI.CyclingSettingManager.SetSetting(SMC.CycylingTime, NewValue);
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}