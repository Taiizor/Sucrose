using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSDEAET = Sucrose.Shared.Dependency.Enum.ApplicationEngineType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEGET = Sucrose.Shared.Dependency.Enum.GifEngineType;
using SSDEUET = Sucrose.Shared.Dependency.Enum.UrlEngineType;
using SSDEVET = Sucrose.Shared.Dependency.Enum.VideoEngineType;
using SSDEWET = Sucrose.Shared.Dependency.Enum.WebEngineType;
using SSDEYTET = Sucrose.Shared.Dependency.Enum.YouTubeEngineType;
using SSLMM = Sucrose.Shared.Live.Manage.Manager;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class WallpaperSettingViewModel : ObservableObject, INavigationAware, IDisposable
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
            TextBlock EngineArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Motorlar"
            };

            Contents.Add(EngineArea);

            SPVCEC UrlPlayer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            UrlPlayer.Title.Text = "Url Motoru";
            UrlPlayer.LeftIcon.Symbol = SymbolRegular.WindowDevTools24;
            UrlPlayer.Description.Text = "Kütüphanenizde bulunan Url türündeki temaları çalıştıracak olan motoru seçin.";

            ComboBox UrlEngine = new();

            UrlEngine.SelectionChanged += (s, e) => UrlEngineSelected(UrlEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEUET Type in Enum.GetValues(typeof(SSDEUET)))
            {
                UrlEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEUET)SSLMM.UApp,
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

            WebPlayer.Title.Text = "Web Motoru";
            WebPlayer.LeftIcon.Symbol = SymbolRegular.WindowDevTools24;
            WebPlayer.Description.Text = "Kütüphanenizde bulunan Web türündeki temaları çalıştıracak olan motoru seçin.";

            ComboBox WebEngine = new();

            WebEngine.SelectionChanged += (s, e) => WebEngineSelected(WebEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEWET Type in Enum.GetValues(typeof(SSDEWET)))
            {
                WebEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEWET)SSLMM.WApp,
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

            YouTubePlayer.Title.Text = "YouTube Motoru";
            YouTubePlayer.LeftIcon.Symbol = SymbolRegular.VideoRecording20;
            YouTubePlayer.Description.Text = "Kütüphanenizde bulunan YouTube türündeki temaları çalıştıracak olan motoru seçin.";

            ComboBox YouTubeEngine = new();

            YouTubeEngine.SelectionChanged += (s, e) => YouTubeEngineSelected(YouTubeEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEYTET Type in Enum.GetValues(typeof(SSDEYTET)))
            {
                YouTubeEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEYTET)SSLMM.YApp,
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

            ApplicationPlayer.Title.Text = "Application Motoru";
            ApplicationPlayer.LeftIcon.Symbol = SymbolRegular.AppGeneric24;
            ApplicationPlayer.Description.Text = "Kütüphanenizde bulunan Application türündeki temaları çalıştıracak olan motoru seçin.";

            ComboBox ApplicationEngine = new();

            ApplicationEngine.SelectionChanged += (s, e) => ApplicationEngineSelected(ApplicationEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEAET Type in Enum.GetValues(typeof(SSDEAET)))
            {
                ApplicationEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEAET)SSLMM.AApp,
                    Content = $"{Type}"
                });
            }

            ApplicationPlayer.HeaderFrame = ApplicationEngine;

            Contents.Add(ApplicationPlayer);

            TextBlock ExtensionArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Eklentiler"
            };

            Contents.Add(ExtensionArea);

            SPVCEC GifPlayer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            GifPlayer.Title.Text = "Gif Eklentisi";
            GifPlayer.LeftIcon.Symbol = SymbolRegular.Gif24;
            GifPlayer.Description.Text = "Kütüphanenizde bulunan Gif türündeki temaları oynatacak olan eklentiyi seçin.";

            ComboBox GifEngine = new();

            GifEngine.SelectionChanged += (s, e) => GifEngineSelected(GifEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEGET Type in Enum.GetValues(typeof(SSDEGET)))
            {
                GifEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEGET)SSLMM.GApp,
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

            VideoPlayer.Title.Text = "Video Eklentisi";
            VideoPlayer.LeftIcon.Symbol = SymbolRegular.VideoClip24;
            VideoPlayer.Description.Text = "Kütüphanenizde bulunan Video türündeki temaları oynatacak olan eklentiyi seçin.";

            ComboBox VideoEngine = new();

            VideoEngine.SelectionChanged += (s, e) => VideoEngineSelected(VideoEngine.SelectedItem as ComboBoxItem);

            foreach (SSDEVET Type in Enum.GetValues(typeof(SSDEVET)))
            {
                VideoEngine.Items.Add(new ComboBoxItem()
                {
                    IsSelected = Type == (SSDEVET)SSLMM.VApp,
                    Content = $"{Type}"
                });
            }

            VideoPlayer.HeaderFrame = VideoEngine;

            Contents.Add(VideoPlayer);

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

        private void GifEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEGET Type) && (SSDEET)Type != SSLMM.GApp)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.GApp, Type);
            }
        }

        private void UrlEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEUET Type) && (SSDEET)Type != SSLMM.UApp)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.UApp, Type);
            }
        }

        private void WebEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEWET Type) && (SSDEET)Type != SSLMM.WApp)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.WApp, Type);
            }
        }

        private void VideoEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEVET Type) && (SSDEET)Type != SSLMM.VApp)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.VApp, Type);
            }
        }

        private void YouTubeEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEYTET Type) && (SSDEET)Type != SSLMM.YApp)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.YApp, Type);
            }
        }

        private void ApplicationEngineSelected(ComboBoxItem Item)
        {
            if (Enum.TryParse($"{Item.Content}", out SSDEAET Type) && (SSDEET)Type != SSLMM.AApp)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.AApp, Type);
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