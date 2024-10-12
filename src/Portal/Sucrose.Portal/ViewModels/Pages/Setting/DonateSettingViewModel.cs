﻿using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SMMCD = Sucrose.Memory.Manage.Constant.Donate;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SRER = Sucrose.Resources.Extension.Resources;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class DonateSettingViewModel : ViewModel, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public DonateSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        private void InitializeViewModel()
        {
            TextBlock DonateArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Donate"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(DonateArea);

            SPVCEC DonateMenu = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            DonateMenu.LeftIcon.Symbol = SymbolRegular.BuildingRetailMoney24;
            DonateMenu.Title.Text = SRER.GetValue("Portal", "DonateSettingPage", "DonateMenu");
            DonateMenu.Description.Text = SRER.GetValue("Portal", "DonateSettingPage", "DonateMenu", "Description");

            ComboBox DonateVisible = new();

            DonateVisible.SelectionChanged += (s, e) => DonateVisibleSelected(DonateVisible.SelectedIndex);

            DonateVisible.Items.Add(SRER.GetValue("Portal", "DonateSettingPage", "DonateMenu", "DonateVisible", "Show"));
            DonateVisible.Items.Add(SRER.GetValue("Portal", "DonateSettingPage", "DonateMenu", "DonateVisible", "Hide"));

            DonateVisible.SelectedIndex = SMMM.DonateVisible ? 0 : 1;

            DonateMenu.HeaderFrame = DonateVisible;

            Contents.Add(DonateMenu);

            TextBlock SupportArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Support"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(SupportArea);

            SPVCEC Advertising = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            Advertising.LeftIcon.Symbol = SymbolRegular.ReceiptMoney24;
            Advertising.Title.Text = SRER.GetValue("Portal", "DonateSettingPage", "Advertising");
            Advertising.Description.Text = SRER.GetValue("Portal", "DonateSettingPage", "Advertising", "Description");

            ToggleSwitch AdvertisingState = new()
            {
                IsChecked = SMMM.AdvertisingState
            };

            AdvertisingState.Checked += (s, e) => AdvertisingStateChecked(true);
            AdvertisingState.Unchecked += (s, e) => AdvertisingStateChecked(false);

            Advertising.HeaderFrame = AdvertisingState;

            StackPanel AdvertisingContent = new();

            StackPanel AdvertisingCustomContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock AdvertisingDelayText = new()
            {
                Text = SRER.GetValue("Portal", "DonateSettingPage", "Advertising", "AdvertisingDelay"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox AdvertisingDelay = new()
            {
                Icon = new SymbolIcon(SymbolRegular.Timer20),
                IconPlacement = ElementPlacement.Left,
                Value = SMMM.AdvertisingDelay,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                MaxLength = 3,
                Maximum = 720,
                Minimum = 30
            };

            AdvertisingDelay.ValueChanged += (s, e) => AdvertisingDelayChanged(AdvertisingDelay.Value);

            TextBlock AdvertisingHint = new()
            {
                Text = SRER.GetValue("Portal", "DonateSettingPage", "Advertising", "AdvertisingHint"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                Margin = new Thickness(0, 10, 0, 0),
                TextAlignment = TextAlignment.Left,
                FontWeight = FontWeights.SemiBold
            };

            AdvertisingCustomContent.Children.Add(AdvertisingDelayText);
            AdvertisingCustomContent.Children.Add(AdvertisingDelay);

            AdvertisingContent.Children.Add(AdvertisingCustomContent);
            AdvertisingContent.Children.Add(AdvertisingHint);

            Advertising.FooterCard = AdvertisingContent;

            Contents.Add(Advertising);

            _isInitialized = true;
        }

        private void DonateVisibleSelected(int Index)
        {
            if (Index != (SMMM.DonateVisible ? 0 : 1))
            {
                bool State = Index == 0;
                Visibility Visible = State ? Visibility.Visible : Visibility.Collapsed;

                SMMI.DonateSettingManager.SetSetting(SMMCD.DonateVisible, State);

                SPMI.DonateService.DonateVisibility = Visible;
            }
        }

        private void AdvertisingStateChecked(bool State)
        {
            SMMI.DonateSettingManager.SetSetting(SMMCD.AdvertisingState, State);
        }

        private void AdvertisingDelayChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.AdvertisingDelay)
            {
                SMMI.DonateSettingManager.SetSetting(SMMCD.AdvertisingDelay, NewValue);
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