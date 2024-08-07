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
using MessageBox = Wpf.Ui.Controls.MessageBox;
using OpenFileDialog = Microsoft.Win32.OpenFileDialog;
using SEOST = Skylark.Enum.OperatingSystemType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDESCT = Sucrose.Shared.Dependency.Enum.SchedulerCommandsType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSIL = Sucrose.Signal.Interface.Launcher;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
using SSSHC = Sucrose.Shared.Space.Helper.Copy;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class GeneralSettingViewModel : ViewModel, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public GeneralSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        public void RefreshInitializeViewModel()
        {
            InitializeViewModel();
        }

        private void InitializeViewModel()
        {
            TextBlock AppearanceBehaviorArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "AppearanceBehavior"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(AppearanceBehaviorArea);

            SPVCEC ApplicationLanguage = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ApplicationLanguage.LeftIcon.Symbol = SymbolRegular.LocalLanguage24;
            ApplicationLanguage.Title.Text = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationLanguage");
            ApplicationLanguage.Description.Text = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationLanguage", "Description");

            ComboBox Localization = new()
            {
                MaxDropDownHeight = 200
            };

            DynamicScrollViewer.SetVerticalScrollBarVisibility(Localization, ScrollBarVisibility.Auto);

            Localization.SelectionChanged += (s, e) => LocalizationSelected(Localization.SelectedIndex);

            foreach (string Code in SRHR.ListLanguage())
            {
                Localization.Items.Add(SRER.GetValue("Locale", Code));
            }

            Localization.SelectedValue = SRER.GetValue("Locale", SMMM.Culture.ToUpperInvariant());

            ApplicationLanguage.HeaderFrame = Localization;

            Contents.Add(ApplicationLanguage);

            SPVCEC ApplicationStartup = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ApplicationStartup.LeftIcon.Symbol = SymbolRegular.Play24;
            ApplicationStartup.Title.Text = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup");
            ApplicationStartup.Description.Text = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Description");

            ComboBox Startup = new();

            Startup.SelectionChanged += (s, e) => StartupSelected(Startup, Startup.SelectedIndex);

            Startup.Items.Add(SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Startup", "None"));
            Startup.Items.Add(SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Startup", "Normal"));
            Startup.Items.Add(SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Startup", "Priority"));
            Startup.Items.Add(SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Startup", "Scheduler"));

            Startup.SelectedIndex = SMMM.Startup;

            ApplicationStartup.HeaderFrame = Startup;

            Contents.Add(ApplicationStartup);

            SPVCEC NotifyIcon = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            NotifyIcon.LeftIcon.Symbol = SymbolRegular.TrayItemAdd24;
            NotifyIcon.Title.Text = SRER.GetValue("Portal", "GeneralSettingPage", "NotifyIcon");
            NotifyIcon.Description.Text = SRER.GetValue("Portal", "GeneralSettingPage", "NotifyIcon", "Description");

            ComboBox Notify = new();

            Notify.SelectionChanged += (s, e) => NotifySelected(Notify.SelectedIndex);

            Notify.Items.Add(SRER.GetValue("Portal", "GeneralSettingPage", "NotifyIcon", "Notify", "Show"));
            Notify.Items.Add(SRER.GetValue("Portal", "GeneralSettingPage", "NotifyIcon", "Notify", "Hide"));

            Notify.SelectedIndex = SMMM.Visible ? 0 : 1;

            NotifyIcon.HeaderFrame = Notify;

            CheckBox NotifyExit = new()
            {
                Content = SRER.GetValue("Portal", "GeneralSettingPage", "NotifyIcon", "NotifyExit"),
                IsChecked = SMMM.Exit
            };

            NotifyExit.Checked += (s, e) => NotifyExitChecked(true);
            NotifyExit.Unchecked += (s, e) => NotifyExitChecked(false);

            NotifyIcon.FooterCard = NotifyExit;

            Contents.Add(NotifyIcon);

            SPVCEC WindowBackdrop = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            WindowBackdrop.LeftIcon.Symbol = SymbolRegular.ColorBackground24;
            WindowBackdrop.Title.Text = SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop");
            WindowBackdrop.Description.Text = SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "Description");

            ComboBox Backdrop = new();

            Backdrop.SelectionChanged += (s, e) => BackdropSelected(Backdrop.SelectedIndex);

            foreach (WindowBackdropType Type in Enum.GetValues(typeof(WindowBackdropType)))
            {
                Backdrop.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "WindowBackdropType", $"{Type}"),
                    IsEnabled = WindowBackdropSupport(Type)
                });
            }

            Backdrop.SelectedIndex = (int)SPMM.BackdropType;

            WindowBackdrop.HeaderFrame = Backdrop;

            StackPanel BackdropContent = new();

            StackPanel BackdropImageContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            Button BackgroundImage = new()
            {
                Content = string.IsNullOrEmpty(SMMM.BackgroundImage) ? SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "BackgroundImage", "Select") : SMMM.BackgroundImage,
                Cursor = Cursors.Hand,
                MaxWidth = 700,
                MinWidth = 350
            };

            BackgroundImage.Click += (s, e) => BackgroundImageClick(BackgroundImage);

            SymbolIcon BackgroundRemove = new()
            {
                Symbol = SymbolRegular.DeleteDismiss24,
                FontSize = 28,
                Height = 32,
                Width = 32
            };

            Button BackgroundImageRemove = new()
            {
                Cursor = Cursors.Hand,
                Content = BackgroundRemove,
                Padding = new Thickness(0),
                Margin = new Thickness(10, 0, 0, 0),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush")
            };

            BackgroundImageRemove.Click += (s, e) => BackgroundImageRemoveClick(BackgroundImage);

            StackPanel BackdropCustomContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock BackdropStretchText = new()
            {
                Text = SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "BackdropStretch"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ComboBox BackdropStretch = new();

            BackdropStretch.SelectionChanged += (s, e) => BackdropStretchSelected(BackdropStretch.SelectedIndex);

            foreach (Stretch Type in Enum.GetValues(typeof(Stretch)))
            {
                BackdropStretch.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "Stretch", $"{Type}")
                });
            }

            BackdropStretch.SelectedIndex = (int)SSDMM.BackgroundStretch;

            TextBlock BackdropOpacityText = new()
            {
                Text = SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "BackdropOpacity"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(20, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox BackdropOpacity = new()
            {
                Icon = new SymbolIcon(SymbolRegular.PositionBackward24),
                IconPlacement = ElementPlacement.Left,
                Value = SMMM.BackgroundOpacity,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                MaxLength = 3,
                Maximum = 100,
                Minimum = 0
            };

            BackdropOpacity.ValueChanged += (s, e) => BackdropOpacityChanged(BackdropOpacity.Value);

            BackdropImageContent.Children.Add(BackgroundImage);
            BackdropImageContent.Children.Add(BackgroundImageRemove);

            BackdropCustomContent.Children.Add(BackdropStretchText);
            BackdropCustomContent.Children.Add(BackdropStretch);
            BackdropCustomContent.Children.Add(BackdropOpacityText);
            BackdropCustomContent.Children.Add(BackdropOpacity);

            BackdropContent.Children.Add(BackdropImageContent);
            BackdropContent.Children.Add(BackdropCustomContent);

            WindowBackdrop.FooterCard = BackdropContent;

            Contents.Add(WindowBackdrop);

            TextBlock SoundArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Sound"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(SoundArea);

            SPVCEC EngineVolume = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            EngineVolume.LeftIcon.Symbol = VolumeSymbol(SMMM.Volume);
            EngineVolume.Title.Text = SRER.GetValue("Portal", "GeneralSettingPage", "EngineVolume");
            EngineVolume.Description.Text = SRER.GetValue("Portal", "GeneralSettingPage", "EngineVolume", "Description");

            Slider Volume = new()
            {
                AutoToolTipPlacement = AutoToolTipPlacement.TopLeft,
                TickPlacement = TickPlacement.Both,
                IsSelectionRangeEnabled = false,
                IsMoveToPointEnabled = true,
                IsSnapToTickEnabled = true,
                Value = SMMM.Volume,
                TickFrequency = 1,
                Maximum = 100,
                Minimum = 0,
                Width = 150
            };

            Volume.ValueChanged += (s, e) => VolumeChanged(EngineVolume, Volume.Value);

            EngineVolume.HeaderFrame = Volume;

            StackPanel VolumeContent = new()
            {
                Orientation = Orientation.Vertical
            };

            StackPanel VolumeCustomContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            CheckBox VolumeDesktop = new()
            {
                Content = SRER.GetValue("Portal", "GeneralSettingPage", "EngineVolume", "VolumeDesktop"),
                IsChecked = SMMM.VolumeDesktop
            };

            VolumeDesktop.Checked += (s, e) => VolumeDesktopChecked(true);
            VolumeDesktop.Unchecked += (s, e) => VolumeDesktopChecked(false);

            CheckBox VolumeActive = new()
            {
                Content = SRER.GetValue("Portal", "GeneralSettingPage", "EngineVolume", "VolumeActive"),
                IsChecked = SMMM.VolumeActive
            };

            VolumeActive.Checked += (s, e) => VolumeActiveChecked(true);
            VolumeActive.Unchecked += (s, e) => VolumeActiveChecked(false);

            TextBlock VolumeSensitivityText = new()
            {
                Text = SRER.GetValue("Portal", "GeneralSettingPage", "EngineVolume", "VolumeSensitivity"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(20, 0, 10, 0)
            };

            //Slider VolumeSensitivity = new()
            //{
            //    AutoToolTipPlacement = AutoToolTipPlacement.TopLeft,
            //    TickPlacement = TickPlacement.Both,
            //    IsSelectionRangeEnabled = false,
            //    Value = SMMM.VolumeSensitivity,
            //    IsMoveToPointEnabled = true,
            //    IsSnapToTickEnabled = true,
            //    TickFrequency = 1,
            //    Maximum = 10,
            //    Minimum = 1,
            //    Width = 120
            //};

            NumberBox VolumeSensitivity2 = new()
            {
                Icon = new SymbolIcon(SymbolRegular.Timer24),
                IconPlacement = ElementPlacement.Left,
                Value = SMMM.VolumeSensitivity,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                MaxLength = 2,
                Maximum = 10,
                Minimum = 1
            };

            //VolumeSensitivity.ValueChanged += (s, e) => VolumeSensitivityChanged(VolumeSensitivity.Value);
            VolumeSensitivity2.ValueChanged += (s, e) => VolumeSensitivity2Changed(VolumeSensitivity2.Value);

            VolumeCustomContent.Children.Add(VolumeActive);
            VolumeCustomContent.Children.Add(VolumeSensitivityText);
            //VolumeCustomContent.Children.Add(VolumeSensitivity);
            VolumeCustomContent.Children.Add(VolumeSensitivity2);

            VolumeContent.Children.Add(VolumeDesktop);
            VolumeContent.Children.Add(VolumeCustomContent);

            EngineVolume.FooterCard = VolumeContent;

            Contents.Add(EngineVolume);

            TextBlock LibraryArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Library"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(LibraryArea);

            SPVCEC PrivateLibrary = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            PrivateLibrary.LeftIcon.Symbol = SymbolRegular.Folder24;
            PrivateLibrary.Title.Text = SRER.GetValue("Portal", "GeneralSettingPage", "PrivateLibrary");
            PrivateLibrary.Description.Text = SRER.GetValue("Portal", "GeneralSettingPage", "PrivateLibrary", "Description");

            StackPanel LibraryContent = new();

            StackPanel LibraryLocationContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            Button LibraryLocation = new()
            {
                Content = SMMM.LibraryLocation,
                Cursor = Cursors.Hand,
                MaxWidth = 700,
                MinWidth = 350
            };

            LibraryLocation.Click += (s, e) => LibraryLocationClick(LibraryLocation);

            SymbolIcon LibraryOpen = new()
            {
                Symbol = SymbolRegular.FolderOpen24,
                FontSize = 28,
                Height = 32,
                Width = 32
            };

            Button LibraryLocationOpen = new()
            {
                Cursor = Cursors.Hand,
                Content = LibraryOpen,
                Padding = new Thickness(0),
                Margin = new Thickness(10, 0, 0, 0),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush")
            };

            LibraryLocationOpen.Click += (s, e) => LibraryLocationOpenClick(LibraryLocation);

            CheckBox LibraryMove = new()
            {
                Content = SRER.GetValue("Portal", "GeneralSettingPage", "PrivateLibrary", "LibraryMove"),
                Margin = new Thickness(0, 10, 0, 0),
                IsChecked = SMMM.LibraryMove
            };

            LibraryMove.Checked += (s, e) => LibraryMoveChecked(true);
            LibraryMove.Unchecked += (s, e) => LibraryMoveChecked(false);

            LibraryLocationContent.Children.Add(LibraryLocation);
            LibraryLocationContent.Children.Add(LibraryLocationOpen);

            LibraryContent.Children.Add(LibraryLocationContent);
            LibraryContent.Children.Add(LibraryMove);

            PrivateLibrary.FooterCard = LibraryContent;

            Contents.Add(PrivateLibrary);

            _isInitialized = true;
        }

        private void NotifySelected(int Index)
        {
            if (Index != (SMMM.Visible ? 0 : 1))
            {
                bool State = Index == 0;

                SMMI.LauncherSettingManager.SetSetting(SMC.Visible, State);

                if (SSSHP.Work(SMR.Launcher))
                {
                    if (State)
                    {
                        SSMI.LauncherManager.FileSave<SSIL>(new() { Show = true });
                    }
                    else
                    {
                        SSMI.LauncherManager.FileSave<SSIL>(new() { Hide = true });
                    }
                }
            }
        }

        private void BackdropSelected(int Index)
        {
            if (Index != (int)SPMM.BackdropType)
            {
                ApplicationTheme Theme = ApplicationTheme.Dark;
                WindowBackdropType Type = (WindowBackdropType)Index;

                if (SSDMM.ThemeType == SEWTT.Light)
                {
                    Theme = ApplicationTheme.Light;
                }

                ApplicationThemeManager.Apply(Theme, Type, true);
                WindowBackdrop.ApplyBackdrop(Application.Current.MainWindow, Type);
                WindowBackgroundManager.UpdateBackground(Application.Current.MainWindow, Theme, Type);

                SMMI.PortalSettingManager.SetSetting(SMC.BackdropType, Type);
            }
        }

        private void NotifyExitChecked(bool State)
        {
            SMMI.LauncherSettingManager.SetSetting(SMC.Exit, State);
        }

        private void LibraryMoveChecked(bool State)
        {
            SMMI.LibrarySettingManager.SetSetting(SMC.LibraryMove, State);
        }

        private void LocalizationSelected(int Index)
        {
            string NewCulture = SRHR.ListLanguage()[Index];

            if (NewCulture != SMMM.Culture)
            {
                SRHR.SetLanguage(NewCulture);
                SMMI.GeneralSettingManager.SetSetting(SMC.Culture, NewCulture);

                SPMI.CultureService.CultureCode = NewCulture;
            }
        }

        private void VolumeActiveChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMC.VolumeActive, State);
        }

        private void VolumeDesktopChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMC.VolumeDesktop, State);
        }

        private void BackdropStretchSelected(int Index)
        {
            Stretch NewStretch = (Stretch)Index;

            if (NewStretch != SSDMM.BackgroundStretch)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.BackgroundStretch, NewStretch);

                SPMI.BackdropService.BackdropStretch = NewStretch;
            }
        }

        private SymbolRegular VolumeSymbol(double Value)
        {
            if (Value <= 0d)
            {
                return SymbolRegular.SpeakerMute24;
            }
            else if (Value >= 75d)
            {
                return SymbolRegular.Speaker224;
            }
            else if (Value >= 25d)
            {
                return SymbolRegular.Speaker124;
            }
            else
            {
                return SymbolRegular.Speaker024;
            }
        }

        private void BackdropOpacityChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.BackgroundOpacity)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.BackgroundOpacity, NewValue);

                SPMI.BackdropService.BackdropOpacity = NewValue;
            }
        }

        //private void VolumeSensitivityChanged(double Value)
        //{
        //    SMMI.EngineSettingManager.SetSetting(SMC.VolumeSensitivity, Convert.ToInt32(Value));
        //}

        private void VolumeSensitivity2Changed(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.VolumeSensitivity)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.VolumeSensitivity, NewValue);
            }
        }

        private void VolumeChanged(SPVCEC Volume, double Value)
        {
            Volume.LeftIcon.Symbol = VolumeSymbol(Value);

            SMMI.EngineSettingManager.SetSetting(SMC.Volume, Convert.ToInt32(Value));
        }

        private async void BackgroundImageClick(Button BackgroundImage)
        {
            BackgroundImage.IsEnabled = false;

            string Startup = string.IsNullOrEmpty(SMMM.BackgroundImage) ? SMR.DesktopPath : Path.GetDirectoryName(SMMM.BackgroundImage);

            OpenFileDialog FileDialog = new()
            {
                Filter = SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "BackgroundImage", "Filter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "BackgroundImage", "Title"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true && !string.IsNullOrEmpty(FileDialog.FileName))
            {
                string Destination = FileDialog.FileName;

                if (Destination != SMMM.BackgroundImage)
                {
                    if (SSSHA.File(Destination))
                    {
                        BackgroundImage.Content = Destination;

                        SMMI.PortalSettingManager.SetSetting(SMC.BackgroundImage, Destination);

                        SPMI.BackdropService.BackdropImage = Destination;
                    }
                    else
                    {
                        BackgroundImage.Content = SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "BackgroundImage", "Access");

                        await Task.Delay(3000);

                        BackgroundImage.Content = string.IsNullOrEmpty(SMMM.BackgroundImage) ? SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "BackgroundImage", "Select") : SMMM.BackgroundImage;
                    }
                }
            }

            BackgroundImage.IsEnabled = true;
        }

        private async void LibraryLocationClick(Button LibraryLocation)
        {
            LibraryLocation.IsEnabled = false;

            FolderBrowserDialog BrowserDialog = new()
            {
                ShowNewFolderButton = true,

                SelectedPath = SMMM.LibraryLocation
            };

            if (BrowserDialog.ShowDialog() == DialogResult.OK && !string.IsNullOrEmpty(BrowserDialog.SelectedPath))
            {
                string Destination = BrowserDialog.SelectedPath;

                if (Destination != SMMM.LibraryLocation)
                {
                    if (SSSHA.Directory(Destination))
                    {
                        if (!SMMM.LibraryMove || (!Directory.GetFiles(Destination).Any() && !Directory.GetDirectories(Destination).Any()))
                        {
                            LibraryLocation.Content = SRER.GetValue("Portal", "GeneralSettingPage", "PrivateLibrary", "LibraryLocation", "Move");

                            if (SMMM.LibraryMove)
                            {
                                if (SSSHL.Run())
                                {
                                    SSLHK.Stop();

                                    await Task.Delay(500);

                                    await Task.Run(() => SSSHC.Folder(SMMM.LibraryLocation, Destination));

                                    await Task.Delay(500);

                                    SMMI.LibrarySettingManager.SetSetting(SMC.LibraryLocation, Destination);

                                    SSLHR.Start();
                                }
                                else
                                {
                                    await Task.Run(() => SSSHC.Folder(SMMM.LibraryLocation, Destination));

                                    await Task.Delay(500);

                                    SMMI.LibrarySettingManager.SetSetting(SMC.LibraryLocation, Destination);
                                }
                            }
                            else
                            {
                                if (SSSHL.Run())
                                {
                                    SSLHK.Stop();
                                }

                                SMMI.LibrarySettingManager.SetSetting(SMC.LibraryLocation, Destination);
                            }

                            LibraryLocation.Content = Destination;
                        }
                        else
                        {
                            LibraryLocation.Content = SRER.GetValue("Portal", "GeneralSettingPage", "PrivateLibrary", "LibraryLocation", "Empty");

                            await Task.Delay(3000);

                            LibraryLocation.Content = SMMM.LibraryLocation;
                        }
                    }
                    else
                    {
                        LibraryLocation.Content = SRER.GetValue("Portal", "GeneralSettingPage", "PrivateLibrary", "LibraryLocation", "Access");

                        await Task.Delay(3000);

                        LibraryLocation.Content = SMMM.LibraryLocation;
                    }
                }
            }

            LibraryLocation.IsEnabled = true;
        }

        private async void StartupSelected(ComboBox Startup, int Index)
        {
            if (Index != SMMM.Startup)
            {
                switch (SMMM.Startup)
                {
                    case 1:
                        SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Startup}{SMR.ValueSeparator}{SMR.AppName}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{false}");
                        break;
                    case 2:
                        try
                        {
                            SSSHP.Runas(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.StartupP}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{false}");
                        }
                        catch (Exception Exception)
                        {
                            await SSWW.Watch_CatchException(Exception);
                            Startup.SelectedIndex = SMMM.Startup;

                            MessageBox Warning = new()
                            {
                                Title = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Warning", "Title"),
                                Content = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Warning", "Message"),
                                CloseButtonText = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Warning", "Close")
                            };

                            await Warning.ShowDialogAsync();

                            return;
                        }
                        break;
                    case 3:
                        SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Scheduler}{SMR.ValueSeparator}{SSDESCT.Delete}");
                        break;
                    default:
                        break;
                }

                switch (Index)
                {
                    case 1:
                        SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Startup}{SMR.ValueSeparator}{SMR.AppName}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{true}");
                        break;
                    case 2:
                        try
                        {
                            SSSHP.Runas(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.StartupP}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{true}");
                        }
                        catch (Exception Exception)
                        {
                            await SSWW.Watch_CatchException(Exception);
                            Startup.SelectedIndex = SMMM.Startup;

                            MessageBox Warning = new()
                            {
                                Title = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Warning", "Title"),
                                Content = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Warning", "Message"),
                                CloseButtonText = SRER.GetValue("Portal", "GeneralSettingPage", "ApplicationStartup", "Warning", "Close")
                            };

                            await Warning.ShowDialogAsync();

                            return;
                        }
                        break;
                    case 3:
                        SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Scheduler}{SMR.ValueSeparator}{SSDESCT.Create}{SMR.ValueSeparator}{SSSMI.Launcher}");
                        break;
                    default:
                        break;
                }

                SMMI.GeneralSettingManager.SetSetting(SMC.Startup, Index);
            }
        }

        private void LibraryLocationOpenClick(Button LibraryLocation)
        {
            if (LibraryLocation.IsEnabled)
            {
                string Destination = $"{LibraryLocation.Content}";

                if (!Directory.Exists(Destination))
                {
                    Directory.CreateDirectory(Destination);
                }

                SSSHP.Run(Destination);
            }
        }

        private void BackgroundImageRemoveClick(Button BackgroundImage)
        {
            BackgroundImage.Content = SRER.GetValue("Portal", "GeneralSettingPage", "WindowBackdrop", "BackgroundImage", "Select");

            SMMI.PortalSettingManager.SetSetting(SMC.BackgroundImage, string.Empty);

            SPMI.BackdropService.BackdropImage = string.Empty;
        }

        private bool WindowBackdropSupport(WindowBackdropType Backdrop)
        {
            if ((SSCHOS.Get() == SEOST.Windows11 || Backdrop == WindowBackdropType.None) && WindowBackdrop.IsSupported(Backdrop))
            {
                return true;
            }
            else
            {
                return false;
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