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
    public partial class GeneralSettingViewModel : ObservableObject, INavigationAware, IDisposable
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
            TextBlock AppearanceBehavior = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Görünüş & Davranış"
            };

            Contents.Add(AppearanceBehavior);

            SPVCEC ApplicationLanguage = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ApplicationLanguage.Title.Text = "Uygulama Dili";
            ApplicationLanguage.LeftIcon.Symbol = SymbolRegular.LocalLanguage24;
            ApplicationLanguage.Description.Text = "Uygulamayı görüntülemek istediğiniz dili seçin.";

            ComboBox Localization = new();

            Localization.SelectionChanged += (s, e) => LocalizationSelected(Localization.SelectedIndex);

            foreach (string Code in SSRHR.ListLanguage())
            {
                Localization.Items.Add(SSRER.GetValue("Locale", Code));
            }

            Localization.SelectedValue = SSRER.GetValue("Locale", SPMM.Culture.ToUpperInvariant());

            ApplicationLanguage.HeaderFrame = Localization;

            Contents.Add(ApplicationLanguage);

            SPVCEC ApplicationStartup = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ApplicationStartup.Title.Text = "Başlangıçta Çalıştır";
            ApplicationStartup.LeftIcon.Symbol = SymbolRegular.Play24;
            ApplicationStartup.Description.Text = "Duvar kağıdını oynatabilmek için Sucrose arka planda çalışmalı.";

            ComboBox Startup = new();

            Startup.SelectionChanged += (s, e) => StartupSelected(Startup.SelectedIndex);

            Startup.Items.Add("Yok");
            Startup.Items.Add("Normal");
            Startup.Items.Add("Makine");
            Startup.Items.Add("Öncelik");
            Startup.Items.Add("Zamanlayıcı");

            Startup.SelectedIndex = SPMM.Startup;

            ApplicationStartup.HeaderFrame = Startup;

            Contents.Add(ApplicationStartup);

            SPVCEC NotifyIcon = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            NotifyIcon.Title.Text = "Bildirim Alanı Simgesi";
            NotifyIcon.LeftIcon.Symbol = SymbolRegular.TrayItemAdd24;
            NotifyIcon.Description.Text = "Sistem tepsisi görünürlüğü, Sucrose gizli bir şekilde çalışmaya devam edecek.";

            ComboBox Notify = new();

            Notify.SelectionChanged += (s, e) => NotifySelected(Notify.SelectedIndex);

            Notify.Items.Add("Görünür");
            Notify.Items.Add("Görünmez");

            Notify.SelectedIndex = SPMM.Visible ? 0 : 1;

            NotifyIcon.HeaderFrame = Notify;

            Contents.Add(NotifyIcon);

            SPVCEC WindowBackdrop = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            WindowBackdrop.Title.Text = "Pencere Arka Planı";
            WindowBackdrop.LeftIcon.Symbol = SymbolRegular.ColorBackground24;
            WindowBackdrop.Description.Text = "Uygulama penceresinin arka planını değiştirmek için kullanılır.";

            ComboBox Backdrop = new();

            Backdrop.SelectionChanged += (s, e) => BackdropSelected(Backdrop.SelectedIndex);

            foreach (WindowBackdropType Type in Enum.GetValues(typeof(WindowBackdropType)))
            {
                Backdrop.Items.Add(new ComboBoxItem()
                {
                    IsEnabled = WindowBackdropSupport(Type),
                    Content = $"{Type}"
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
                Content = string.IsNullOrEmpty(SPMM.BackgroundImage) ? "Bir arkaplan resmi seçin" : SPMM.BackgroundImage,
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
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush")
            };

            BackgroundImageRemove.Click += (s, e) => BackgroundImageRemoveClick(BackgroundImage);

            StackPanel BackdropCustomContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock BackdropStretchText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Konumlandırma:",
            };

            ComboBox BackdropStretch = new();

            BackdropStretch.SelectionChanged += (s, e) => BackdropStretchSelected(BackdropStretch.SelectedIndex);

            foreach (Stretch Type in Enum.GetValues(typeof(Stretch)))
            {
                BackdropStretch.Items.Add(new ComboBoxItem()
                {
                    Content = $"{Type}"
                });
            }

            BackdropStretch.SelectedIndex = (int)SPMM.BackgroundStretch;

            TextBlock BackdropOpacityText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(20, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Opaklık (%):",
            };

            NumberBox BackdropOpacity = new()
            {
                Value = SPMM.BackgroundOpacity,
                ClearButtonEnabled = false,
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

            TextBlock Sound = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Ses"
            };

            Contents.Add(Sound);

            SPVCEC EngineVolume = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            EngineVolume.Title.Text = "Ses Düzeyi";
            EngineVolume.LeftIcon.Symbol = VolumeSymbol(SPMM.Volume);
            EngineVolume.Description.Text = "Tüm duvar kağıtları için ses seviyesi.";

            Slider Volume = new()
            {
                TickPlacement = TickPlacement.Both,
                IsSelectionRangeEnabled = false,
                IsMoveToPointEnabled = true,
                IsSnapToTickEnabled = true,
                Value = SPMM.Volume,
                TickFrequency = 1,
                Maximum = 100,
                Minimum = 0,
                Width = 150
            };

            Volume.ValueChanged += (s, e) => VolumeChanged(EngineVolume, Volume.Value);

            EngineVolume.HeaderFrame = Volume;

            CheckBox VolumeDesktop = new()
            {
                Content = "Sesi yalnızca masaüstü odaklandığında oynat",
                IsChecked = SPMM.VolumeDesktop
            };

            VolumeDesktop.Checked += (s, e) => VolumeDesktopChecked(true);
            VolumeDesktop.Unchecked += (s, e) => VolumeDesktopChecked(false);

            EngineVolume.FooterCard = VolumeDesktop;

            Contents.Add(EngineVolume);

            TextBlock Library = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Kütüphane"
            };

            Contents.Add(Library);

            SPVCEC PrivateLibrary = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            PrivateLibrary.Title.Text = "Kütüphane Konumu";
            PrivateLibrary.LeftIcon.Symbol = SymbolRegular.Folder24;
            PrivateLibrary.Description.Text = "Kütüphanenize eklediğiniz temaların saklanacağı konum.";

            StackPanel LibraryContent = new();

            StackPanel LibraryLocationContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            Button LibraryLocation = new()
            {
                Content = SPMM.LibraryLocation,
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
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush")
            };

            LibraryLocationOpen.Click += (s, e) => LibraryLocationOpenClick(LibraryLocation);

            CheckBox LibraryMove = new()
            {
                Content = "Mevcut kütüphanenizi yeni konuma taşı",
                Margin = new Thickness(0, 10, 0, 0),
                IsChecked = SPMM.LibraryMove
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

        public void OnNavigatedTo()
        {
            //
        }

        public void OnNavigatedFrom()
        {
            //Dispose();
        }

        private void NotifySelected(int Index)
        {
            if (Index != (SPMM.Visible ? 0 : 1))
            {
                bool State = Index == 0;

                SMMI.LauncherSettingManager.SetSetting(SMC.Visible, State);

                if (SSSHP.Work(SMR.Launcher))
                {
                    SGSGSS.ChannelCreate($"{SPMM.Host}", SPMM.Port);
                    SGCLLC Client = new(SGSGSS.ChannelInstance);

                    if (State)
                    {
                        SGCSLCS.GetShow(Client);
                    }
                    else
                    {
                        SGCSLCS.GetHide(Client);
                    }
                }
            }
        }

        private void StartupSelected(int Index)
        {
            if (Index != SPMM.Startup)
            {
                SMMI.GeneralSettingManager.SetSetting(SMC.Startup, Index);

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Startup}{SMR.ValueSeparator}{SMR.AppName}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{false}");
                SSSHP.Runas(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.StartupM}{SMR.ValueSeparator}{SMR.AppName}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{false}");
                SSSHP.Runas(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.StartupP}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{false}");
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Scheduler}{SMR.ValueSeparator}{SSDESCT.Delete}");

                switch (Index)
                {
                    case 1:
                        SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Startup}{SMR.ValueSeparator}{SMR.AppName}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{true}");
                        break;
                    case 2:
                        SSSHP.Runas(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.StartupM}{SMR.ValueSeparator}{SMR.AppName}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{true}");
                        break;
                    case 3:
                        SSSHP.Runas(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.StartupP}{SMR.ValueSeparator}{SSSMI.Launcher}{SMR.ValueSeparator}{true}");
                        break;
                    case 4:
                        SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Scheduler}{SMR.ValueSeparator}{SSDESCT.Create}{SMR.ValueSeparator}{SSSMI.Launcher}");
                        break;
                    default:
                        break;
                }
            }
        }

        private void BackdropSelected(int Index)
        {
            if (Index != (int)SPMM.BackdropType)
            {
                ApplicationTheme Theme = ApplicationTheme.Dark;
                WindowBackdropType Type = (WindowBackdropType)Index;

                if (SPMM.Theme == SEWTT.Light)
                {
                    Theme = ApplicationTheme.Light;
                }

                if (Type != WindowBackdropType.None)
                {
                    WindowBackdrop.RemoveBackground(Application.Current.MainWindow);
                }
                else
                {
                    WindowBackdrop.RemoveBackdrop(Application.Current.MainWindow);
                }

                ApplicationThemeManager.Apply(Theme, Type, true, true);
                WindowBackdrop.ApplyBackdrop(Application.Current.MainWindow, Type);
                WindowBackgroundManager.UpdateBackground(Application.Current.MainWindow, Theme, Type, true);

                SMMI.PortalSettingManager.SetSetting(SMC.BackdropType, Type);
            }
        }

        private void LibraryMoveChecked(bool State)
        {
            SMMI.LibrarySettingManager.SetSetting(SMC.LibraryMove, State);
        }

        private void LocalizationSelected(int Index)
        {
            string NewCulture = SSRHR.ListLanguage()[Index];

            if (NewCulture != SPMM.Culture)
            {
                SMMI.GeneralSettingManager.SetSetting(SMC.CultureName, NewCulture);
                SSRHR.SetLanguage(NewCulture);

                SPMI.CultureService.CultureCode = NewCulture;
            }
        }

        private void VolumeDesktopChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMC.VolumeDesktop, State);
        }

        private void BackdropStretchSelected(int Index)
        {
            Stretch NewStretch = (Stretch)Index;

            if (NewStretch != SPMM.BackgroundStretch)
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

            if (NewValue != SPMM.BackgroundOpacity)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.BackgroundOpacity, NewValue);

                SPMI.BackdropService.BackdropOpacity = NewValue;
            }
        }

        private void VolumeChanged(SPVCEC Volume, double Value)
        {
            Volume.LeftIcon.Symbol = VolumeSymbol(Value);

            SMMI.EngineSettingManager.SetSetting(SMC.Volume, Convert.ToInt32(Value));
        }

        private void BackgroundImageClick(Button BackgroundImage)
        {
            string Startup = string.IsNullOrEmpty(SPMM.BackgroundImage) ? SMR.DesktopPath : Path.GetDirectoryName(SPMM.BackgroundImage);

            OpenFileDialog FileDialog = new()
            {
                Filter = "Image files (*.png;*.jpg;*.jpeg;*.tiff;*.webp;*.gif)|*.png;*.jpg;*.jpeg;*.tiff;*.webp;*.gif",
                FilterIndex = 1,

                Title = SSRER.GetValue("Launcher", "SaveDialogTitle"),

                InitialDirectory = Startup
            };

            if (FileDialog.ShowDialog() == true)
            {
                string Destination = FileDialog.FileName;

                BackgroundImage.Content = Destination;

                SMMI.PortalSettingManager.SetSetting(SMC.BackgroundImage, Destination);

                SPMI.BackdropService.BackdropImage = Destination;
            }
        }

        private async void LibraryLocationClick(Button LibraryLocation)
        {
            LibraryLocation.IsEnabled = false;

            FolderBrowserDialog BrowserDialog = new()
            {
                ShowNewFolderButton = true,

                SelectedPath = SPMM.LibraryLocation
            };

            if (BrowserDialog.ShowDialog() == DialogResult.OK && !string.IsNullOrEmpty(BrowserDialog.SelectedPath))
            {
                string Destination = BrowserDialog.SelectedPath;

                if (Destination != SPMM.LibraryLocation)
                {
                    if (!Directory.GetFiles(Destination).Any() && !Directory.GetDirectories(Destination).Any())
                    {
                        LibraryLocation.Content = "Konum değiştirilirken lütfen biraz bekleyin";

                        if (SPMM.LibraryMove)
                        {
                            if (SSSHL.Run())
                            {
                                SSSHL.Kill();

                                if (!string.IsNullOrEmpty(SPMM.App))
                                {
                                    SSSHP.Kill(SPMM.App);
                                }

                                SWUD.RefreshDesktop();

                                await Task.Delay(250);

                                await Task.Run(() => SSSHC.Folder(SPMM.LibraryLocation, Destination));

                                SMMI.LibrarySettingManager.SetSetting(SMC.LibraryLocation, Destination);

                                SMMI.AuroraSettingManager.SetSetting(SMC.App, string.Empty);

                                await Task.Delay(250);

                                SSLHR.Start();
                            }
                            else
                            {
                                await Task.Run(() => SSSHC.Folder(SPMM.LibraryLocation, Destination));

                                await Task.Delay(250);

                                SMMI.LibrarySettingManager.SetSetting(SMC.LibraryLocation, Destination);
                            }
                        }

                        LibraryLocation.Content = Destination;
                    }
                    else
                    {
                        LibraryLocation.Content = "Lütfen boş bir klasör seç";

                        await Task.Delay(1500);

                        LibraryLocation.Content = SPMM.LibraryLocation;
                    }
                }
            }

            LibraryLocation.IsEnabled = true;
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
            BackgroundImage.Content = "Bir arkaplan resmi seçin";

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