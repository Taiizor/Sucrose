﻿using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using System.Windows;
using System.Windows.Threading;
using Wpf.Ui.Controls;
using SEOST = Skylark.Enum.OperatingSystemType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPMM = Sucrose.Portal.Manage.Manager;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHM = Sucrose.Shared.Core.Helper.Memory;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using WUAAT = Wpf.Ui.Appearance.ApplicationTheme;
using WUAT = Wpf.Ui.Appearance.ApplicationThemeManager;

namespace Sucrose.Portal.ViewModels.Windows
{
    public partial class MainWindowViewModel : ObservableObject, IDisposable
    {
        [ObservableProperty]
        private WindowBackdropType _WindowBackdropType = GetWindowBackdropType();

        private readonly DispatcherTimer Timer = new();

        [ObservableProperty]
        private SEOST _OperatingSystem = SEOST.Unknown;

        [ObservableProperty]
        private string _Architecture = string.Empty;

        [ObservableProperty]
        private string _Framework = string.Empty;

        [ObservableProperty]
        private string _Version = string.Empty;

        [ObservableProperty]
        private string _Quoting = string.Empty;

        [ObservableProperty]
        private string _Memory = string.Empty;

        [ObservableProperty]
        private IconElement _ThemeIcon = null;

        private bool _isInitialized;

        public MainWindowViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();

                Timer.Interval = TimeSpan.FromSeconds(1);
                Timer.Tick += Memory_Tick;
                Timer.Start();
            }
        }

        private void InitializeViewModel()
        {
            Memory = SSCHM.Get();
            Quoting = GetQuoting();
            Version = SSCHV.GetText();
            Framework = SSCHF.GetName();
            Architecture = SSCHA.GetText();
            WindowBackdropType = GetWindowBackdropType();

            _isInitialized = true;
        }

        private string GetQuoting()
        {
            return SSRER.GetValue("Portal", $"Quoting{SMR.Randomise.Next(40)}");
        }

        [RelayCommand]
        private void OnChangeTheme()
        {
            if (SPMM.Theme == SEWTT.Dark)
            {
                SMMI.GeneralSettingManager.SetSetting(SMC.ThemeType, SEWTT.Light);
                WUAT.Apply(WUAAT.Light, GetWindowBackdropType(), true, true);
            }
            else
            {
                SMMI.GeneralSettingManager.SetSetting(SMC.ThemeType, SEWTT.Dark);
                WUAT.Apply(WUAAT.Dark, GetWindowBackdropType(), true, true);
            }

            if (GetWindowBackdropType() == WindowBackdropType.None)
            {
                WindowBackdrop.RemoveBackdrop(Application.Current.MainWindow);
            }
        }

        private static WindowBackdropType GetWindowBackdropType()
        {
            if (WindowBackdrop.IsSupported(SPMM.BackdropType))
            {
                return SPMM.BackdropType;
            }
            else
            {
                return SPMM.DefaultBackdropType;
            }
        }

        private void Memory_Tick(object sender, EventArgs e)
        {
            Memory = SSCHM.Get();
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}