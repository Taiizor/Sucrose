using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEIT = Skylark.Enum.InputType;
using SEST = Skylark.Enum.ScreenType;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDEAET = Sucrose.Shared.Dependency.Enum.ApplicationEngineType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEGET = Sucrose.Shared.Dependency.Enum.GifEngineType;
using SSDEIMT = Sucrose.Shared.Dependency.Enum.InputModuleType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSDEUET = Sucrose.Shared.Dependency.Enum.UrlEngineType;
using SSDEVET = Sucrose.Shared.Dependency.Enum.VideoEngineType;
using SSDEWET = Sucrose.Shared.Dependency.Enum.WebEngineType;
using SSDEYTET = Sucrose.Shared.Dependency.Enum.YouTubeEngineType;
using SSDMME = Sucrose.Shared.Dependency.Manage.Manager.Engine;
using SWUD = Skylark.Wing.Utility.Desktop;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class WallpaperSettingViewModel : ViewModel, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public WallpaperSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
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

            SPVCEC InputMode = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true,
                IsExpand = true
            };

            InputMode.LeftIcon.Symbol = InputSymbol(SMME.InputType);
            InputMode.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "InputMode");
            InputMode.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "InputMode", "Description");

            ComboBox InputType = new();

            InputType.SelectionChanged += (s, e) => InputTypeSelected(InputMode, InputType.SelectedIndex);

            foreach (SEIT Type in Enum.GetValues(typeof(SEIT)))
            {
                InputType.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "InputType", $"{Type}")
                });
            }

            InputType.SelectedIndex = (int)SMME.InputType;

            InputMode.HeaderFrame = InputType;

            StackPanel InputContent = new()
            {
                Orientation = Orientation.Vertical
            };

            StackPanel InputModuleContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock InputModuleText = new()
            {
                Text = SRER.GetValue("Portal", "WallpaperSettingPage", "InputMode", "InputModule"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ComboBox InputModuleType = new();

            InputModuleType.SelectionChanged += (s, e) => InputModuleTypeSelected(InputModuleType.SelectedIndex);

            foreach (SSDEIMT Type in Enum.GetValues(typeof(SSDEIMT)))
            {
                InputModuleType.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "InputModuleType", $"{Type}")
                });
            }

            InputModuleType.SelectedIndex = (int)SSDMME.InputModuleType;

            StackPanel InputCustomContent = new()
            {
                Orientation = Orientation.Vertical,
                Margin = new Thickness(0, 10, 0, 0)
            };

            CheckBox InputDesktop = new()
            {
                Content = SRER.GetValue("Portal", "WallpaperSettingPage", "InputMode", "InputDesktop"),
                IsChecked = SMME.InputDesktop
            };

            InputDesktop.Checked += (s, e) => InputDesktopChecked(true);
            InputDesktop.Unchecked += (s, e) => InputDesktopChecked(false);

            CheckBox DesktopIcon = new()
            {
                Content = SRER.GetValue("Portal", "WallpaperSettingPage", "InputMode", "DesktopIcon"),
                IsChecked = !SWUD.GetDesktopIconVisibility(),
                Margin = new Thickness(0, 10, 0, 0)
            };

            DesktopIcon.Checked += (s, e) => DesktopIconChecked(false);
            DesktopIcon.Unchecked += (s, e) => DesktopIconChecked(true);

            InputModuleContent.Children.Add(InputModuleText);
            InputModuleContent.Children.Add(InputModuleType);

            InputCustomContent.Children.Add(InputDesktop);
            InputCustomContent.Children.Add(DesktopIcon);

            InputContent.Children.Add(InputModuleContent);
            InputContent.Children.Add(InputCustomContent);

            InputMode.FooterCard = InputContent;

            Contents.Add(InputMode);

            SPVCEC ScreenLayout = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ScreenLayout.LeftIcon.Symbol = SymbolRegular.DesktopFlow24;
            ScreenLayout.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "ScreenLayout");
            ScreenLayout.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "ScreenLayout", "Description");

            ComboBox ScreenType = new();

            ScreenType.SelectionChanged += (s, e) => ScreenTypeSelected(ScreenType.SelectedIndex);

            foreach (SEST Type in Enum.GetValues(typeof(SEST)))
            {
                ScreenType.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "ScreenType", $"{Type}")
                });
            }

            ScreenType.SelectedIndex = (int)SMME.ScreenType;

            ScreenLayout.HeaderFrame = ScreenType;

            Contents.Add(ScreenLayout);

            SPVCEC StretchMode = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            StretchMode.LeftIcon.Symbol = SymbolRegular.ArrowMinimize24;
            StretchMode.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "StretchMode");
            StretchMode.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "StretchMode", "Description");

            ComboBox StretchType = new();

            StretchType.SelectionChanged += (s, e) => StretchTypeSelected(StretchType.SelectedIndex);

            foreach (SSDEST Type in Enum.GetValues(typeof(SSDEST)))
            {
                StretchType.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "StretchType", $"{Type}")
                });
            }

            StretchType.SelectedIndex = (int)SSDMME.StretchType;

            StretchMode.HeaderFrame = StretchType;

            Contents.Add(StretchMode);

            SPVCEC LoopMode = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            LoopMode.LeftIcon.Symbol = SymbolRegular.ArrowRepeatAll24;
            LoopMode.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "LoopMode");
            LoopMode.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "LoopMode", "Description");

            ToggleSwitch LoopState = new()
            {
                IsChecked = SMME.Loop
            };

            LoopState.Checked += (s, e) => LoopStateChecked(true);
            LoopState.Unchecked += (s, e) => LoopStateChecked(false);

            LoopMode.HeaderFrame = LoopState;

            Contents.Add(LoopMode);

            SPVCEC ShuffleMode = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ShuffleMode.LeftIcon.Symbol = SymbolRegular.ArrowShuffle24;
            ShuffleMode.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "ShuffleMode");
            ShuffleMode.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "ShuffleMode", "Description");

            ToggleSwitch ShuffleState = new()
            {
                IsChecked = SMME.Shuffle
            };

            ShuffleState.Checked += (s, e) => ShuffleStateChecked(true);
            ShuffleState.Unchecked += (s, e) => ShuffleStateChecked(false);

            ShuffleMode.HeaderFrame = ShuffleState;

            Contents.Add(ShuffleMode);

            TextBlock ExtensionsArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Extensions"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(ExtensionsArea);

            SPVCEC GifPlayer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            GifPlayer.LeftIcon.Symbol = SymbolRegular.Gif24;
            GifPlayer.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "GifPlayer");
            GifPlayer.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "GifPlayer", "Description");

            ComboBox GifEngine = new();

            GifEngine.SelectionChanged += (s, e) => GifEngineSelected(GifEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEGET Type in Enum.GetValues(typeof(SSDEGET)))
            {
                GifEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEGET)SSDMME.GifEngine,
                    Content = $"{Type}"
                });
            }

            GifPlayer.HeaderFrame = GifEngine;

            Contents.Add(GifPlayer);

            SPVCEC VideoPlayer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            VideoPlayer.LeftIcon.Symbol = SymbolRegular.VideoClip24;
            VideoPlayer.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "VideoPlayer");
            VideoPlayer.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "VideoPlayer", "Description");

            ComboBox VideoEngine = new();

            VideoEngine.SelectionChanged += (s, e) => VideoEngineSelected(VideoEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEVET Type in Enum.GetValues(typeof(SSDEVET)))
            {
                VideoEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEVET)SSDMME.VideoEngine,
                    Content = $"{Type}"
                });
            }

            VideoPlayer.HeaderFrame = VideoEngine;

            Contents.Add(VideoPlayer);

            TextBlock EnginesArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Engines"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(EnginesArea);

            SPVCEC UrlPlayer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            UrlPlayer.LeftIcon.Symbol = SymbolRegular.SlideLink24;
            UrlPlayer.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "UrlPlayer");
            UrlPlayer.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "UrlPlayer", "Description");

            ComboBox UrlEngine = new();

            UrlEngine.SelectionChanged += (s, e) => UrlEngineSelected(UrlEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEUET Type in Enum.GetValues(typeof(SSDEUET)))
            {
                UrlEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEUET)SSDMME.UrlEngine,
                    Content = $"{Type}"
                });
            }

            UrlPlayer.HeaderFrame = UrlEngine;

            Contents.Add(UrlPlayer);

            SPVCEC WebPlayer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            WebPlayer.LeftIcon.Symbol = SymbolRegular.GlobeDesktop24;
            WebPlayer.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "WebPlayer");
            WebPlayer.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "WebPlayer", "Description");

            ComboBox WebEngine = new();

            WebEngine.SelectionChanged += (s, e) => WebEngineSelected(WebEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEWET Type in Enum.GetValues(typeof(SSDEWET)))
            {
                WebEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEWET)SSDMME.WebEngine,
                    Content = $"{Type}"
                });
            }

            WebPlayer.HeaderFrame = WebEngine;

            Contents.Add(WebPlayer);

            SPVCEC YouTubePlayer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            YouTubePlayer.LeftIcon.Symbol = SymbolRegular.VideoRecording20;
            YouTubePlayer.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "YouTubePlayer");
            YouTubePlayer.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "YouTubePlayer", "Description");

            ComboBox YouTubeEngine = new();

            YouTubeEngine.SelectionChanged += (s, e) => YouTubeEngineSelected(YouTubeEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEYTET Type in Enum.GetValues(typeof(SSDEYTET)))
            {
                YouTubeEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEYTET)SSDMME.YouTubeEngine,
                    Content = $"{Type}"
                });
            }

            YouTubePlayer.HeaderFrame = YouTubeEngine;

            Contents.Add(YouTubePlayer);

            SPVCEC ApplicationPlayer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ApplicationPlayer.LeftIcon.Symbol = SymbolRegular.AppGeneric24;
            ApplicationPlayer.Title.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "ApplicationPlayer");
            ApplicationPlayer.Description.Text = SRER.GetValue("Portal", "WallpaperSettingPage", "ApplicationPlayer", "Description");

            ComboBox ApplicationEngine = new();

            ApplicationEngine.SelectionChanged += (s, e) => ApplicationEngineSelected(ApplicationEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEAET Type in Enum.GetValues(typeof(SSDEAET)))
            {
                ApplicationEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEAET)SSDMME.ApplicationEngine,
                    Content = $"{Type}"
                });
            }

            ApplicationPlayer.HeaderFrame = ApplicationEngine;

            Contents.Add(ApplicationPlayer);

            _isInitialized = true;
        }

        private void LoopStateChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMMCE.Loop, State);
        }

        private void ScreenTypeSelected(int Index)
        {
            SEST NewScreen = (SEST)Index;

            if (NewScreen != SMME.ScreenType)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.ScreenType, NewScreen);
            }
        }

        private void StretchTypeSelected(int Index)
        {
            SSDEST NewStretch = (SSDEST)Index;

            if (NewStretch != SSDMME.StretchType)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.StretchType, NewStretch);
            }
        }

        private void DesktopIconChecked(bool State)
        {
            SWUD.SetDesktopIconVisibility(State);
        }

        private void InputDesktopChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMMCE.InputDesktop, State);
        }

        private SymbolRegular InputSymbol(SEIT Type)
        {
            return Type switch
            {
                SEIT.Close => SymbolRegular.HandDraw24,
                SEIT.OnlyMouse => SymbolRegular.DesktopCursor24,
                SEIT.OnlyKeyboard => SymbolRegular.DesktopKeyboard24,
                SEIT.MouseKeyboard => SymbolRegular.KeyboardMouse16,
                _ => SymbolRegular.Desktop24,
            };
        }

        private void ShuffleStateChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMMCE.Shuffle, State);
        }

        private void InputModuleTypeSelected(int Index)
        {
            SSDEIMT NewInput = (SSDEIMT)Index;

            if (NewInput != SSDMME.InputModuleType)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.InputModuleType, NewInput);
            }
        }

        private void GifEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEGET Type) && (SSDEET)Type != SSDMME.GifEngine)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.GifEngine, (SSDEET)Type);
            }
        }

        private void UrlEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEUET Type) && (SSDEET)Type != SSDMME.UrlEngine)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.UrlEngine, (SSDEET)Type);
            }
        }

        private void WebEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEWET Type) && (SSDEET)Type != SSDMME.WebEngine)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.WebEngine, (SSDEET)Type);
            }
        }

        private void VideoEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEVET Type) && (SSDEET)Type != SSDMME.VideoEngine)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.VideoEngine, (SSDEET)Type);
            }
        }

        private void YouTubeEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEYTET Type) && (SSDEET)Type != SSDMME.YouTubeEngine)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.YouTubeEngine, (SSDEET)Type);
            }
        }

        private void InputTypeSelected(SPVCEC Input, int Index)
        {
            SEIT NewInput = (SEIT)Index;

            if (NewInput != SMME.InputType)
            {
                Input.LeftIcon.Symbol = InputSymbol(NewInput);
                SMMI.EngineSettingManager.SetSetting(SMMCE.InputType, NewInput);
            }
        }

        private void ApplicationEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEAET Type) && (SSDEET)Type != SSDMME.ApplicationEngine)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.ApplicationEngine, (SSDEET)Type);
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