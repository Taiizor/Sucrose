using System.IO;
using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Controls;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDMI = Sucrose.Shared.Dependency.Manage.Internal;
using SSSHQ = Sucrose.Shared.Space.Helper.Lockscreen;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// LockScreenVideo.xaml interaction logic (video to GIF output settings,
    /// download and conversion with cancellation on close).
    /// </summary>
    public partial class LockScreenVideo : ContentDialog, IDisposable
    {
        public string SourcePath { get; set; } = string.Empty;
        public string ThumbnailSource { get; set; }
        public string ResultGifPath { get; private set; }

        private readonly CancellationTokenSource _cancellation = new();
        private readonly SPEIL _loader = new();
        private string _ffmpeg = string.Empty;
        private string _outputDirectory;
        private string _previewDirectory;
        private bool _working;

        public int VideoWidth { get; private set; }
        public int VideoHeight { get; private set; }
        public double VideoFps { get; private set; }
        public double VideoDuration { get; private set; }

        public int SelectedWidth { get; private set; }
        public double SelectedFps { get; private set; }
        public int SelectedSeconds { get; private set; } = 10;
        public bool UseHardware { get; private set; }

        public LockScreenVideo() : base(SPMI.ContentDialogService.GetDialogHostEx())
        {
            InitializeComponent();
        }

        private async void Dialog_Loaded(object sender, RoutedEventArgs e)
        {
            // Download, probe and preview all run here. Closing the dialog
            // cancels whatever stage is in progress through _cancellation.
            _working = true;

            try
            {
                string Destination = SSSHQ.GetFFmpegPath();

                if (!File.Exists(Destination))
                {
                    Progress<double> Download = new(OnDownloadProgress);

                    _ffmpeg = await SSSHQ.EnsureFFmpegAsync(SSDMI.Client, SMMRU.FFmpeg, Destination, Download, _cancellation.Token);
                }
                else
                {
                    _ffmpeg = Destination;
                }

                SetStatus(SRER.GetValue("Portal", "LockScreenVideo", "StatusAnalyze"), true);

                SSSHQ.VideoProbe Probe = await SSSHQ.ProbeVideoAsync(_ffmpeg, SourcePath, _cancellation.Token);

                VideoWidth = Probe.Width;
                VideoHeight = Probe.Height;
                VideoFps = Probe.Fps;
                VideoDuration = Probe.DurationSeconds;

                PreviewName.Text = Path.GetFileName(SourcePath);
                PreviewMeta.Text = BuildMeta();

                Populate();

                await LoadPreviewAsync();

                OptionsPanel.Visibility = Visibility.Visible;
                ProgressPanel.Visibility = Visibility.Collapsed;
                ConvertButton.IsEnabled = true;
                _working = false;
            }
            catch (OperationCanceledException)
            {
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);

                PreviewRing.Visibility = Visibility.Collapsed;
                _working = false;

                SetStatus(string.Format(SRER.GetValue("Portal", "LockScreenVideo", "StatusError"), Exception.Message), false);
            }
        }

        private async Task LoadPreviewAsync()
        {
            // Prefer an actual video frame, fall back to the theme thumbnail.
            try
            {
                try
                {
                    _previewDirectory = Path.Combine(Path.GetTempPath(), $"SucroseLockScreen_{Guid.NewGuid():N}");

                    string Frame = await SSSHQ.ExtractThumbnailAsync(_ffmpeg, SourcePath, _previewDirectory, 560, _cancellation.Token);

                    PreviewImage.Source = await _loader.LoadOptimalAsync(Frame, true, 560);
                    PreviewImage.Visibility = Visibility.Visible;

                    return;
                }
                catch (OperationCanceledException)
                {
                    throw;
                }
                catch (Exception Exception)
                {
                    await SSWEW.Watch_CatchException(Exception);
                }

                try
                {
                    if (!string.IsNullOrEmpty(ThumbnailSource) && File.Exists(ThumbnailSource))
                    {
                        PreviewImage.Source = await _loader.LoadOptimalAsync(ThumbnailSource, true, 560);
                        PreviewImage.Visibility = Visibility.Visible;

                        return;
                    }
                }
                catch (Exception Exception)
                {
                    await SSWEW.Watch_CatchException(Exception);
                }

                PreviewPlaceholder.Visibility = Visibility.Visible;
            }
            finally
            {
                PreviewRing.Visibility = Visibility.Collapsed;
            }
        }

        private async void ConvertButton_Click(object sender, RoutedEventArgs e)
        {
            if (_working || OptionsPanel.Visibility != Visibility.Visible)
            {
                return;
            }

            try
            {
                _working = true;
                ConvertButton.IsEnabled = false;

                ErrorBar.IsOpen = false;
                OptionsPanel.Visibility = Visibility.Collapsed;
                ProgressPanel.Visibility = Visibility.Visible;

                WorkProgress.IsIndeterminate = true;
                WorkProgress.Value = 0;

                Progress<string> Stage = new(Code => SetStatus(Code switch
                {
                    "GifHw" => SRER.GetValue("Portal", "LockScreenVideo", "StatusConvertHw"),
                    "PaletteHw" => SRER.GetValue("Portal", "LockScreenVideo", "StatusAnalyzeHw"),
                    "CpuFallback" => SRER.GetValue("Portal", "LockScreenVideo", "StatusCpuFallback"),
                    "Gif" => SRER.GetValue("Portal", "LockScreenVideo", "StatusConvert"),
                    _ => SRER.GetValue("Portal", "LockScreenVideo", "StatusAnalyze")
                }, true));

                double Total = SelectedSeconds > 0 ? Math.Min(SelectedSeconds, VideoDuration) : VideoDuration;

                Progress<double> Percent = new(Value =>
                {
                    WorkProgress.IsIndeterminate = false;
                    WorkProgress.Value = Value;
                });

                _outputDirectory = Path.Combine(Path.GetTempPath(), $"SucroseLockScreen_{Guid.NewGuid():N}");

                ResultGifPath = await SSSHQ.ConvertVideoToGifAsync(
                    _ffmpeg,
                    SourcePath,
                    _outputDirectory,
                    SelectedWidth,
                    SelectedFps,
                    SelectedSeconds,
                    HardwareCheck.IsChecked == true,
                    _cancellation.Token,
                    Stage,
                    Total > 0 ? Percent : null,
                    Total);

                Hide(ContentDialogResult.Primary);
            }
            catch (OperationCanceledException)
            {
                CleanupOutput();
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);

                _working = false;
                ConvertButton.IsEnabled = true;
                ProgressPanel.Visibility = Visibility.Collapsed;
                OptionsPanel.Visibility = Visibility.Visible;

                ErrorBar.Message = string.Format(SRER.GetValue("Portal", "LockScreenVideo", "StatusError"), Exception.Message);
                ErrorBar.IsOpen = true;
            }
        }

        private void Dialog_Closing(ContentDialog sender, ContentDialogClosingEventArgs args)
        {
            if (_working)
            {
                try
                {
                    _cancellation.Cancel();
                }
                catch
                {
                }
            }
        }

        private void OnDownloadProgress(double Percent)
        {
            if (Percent < 0)
            {
                SetStatus(SRER.GetValue("Portal", "LockScreenVideo", "StatusExtract"), true);
                return;
            }

            WorkProgress.IsIndeterminate = false;
            WorkProgress.Value = Percent;
            SetStatus(string.Format(SRER.GetValue("Portal", "LockScreenVideo", "StatusDownload"), Percent.ToString("0")), false);
        }

        private void SetStatus(string Text, bool Indeterminate)
        {
            StatusText.Text = Text;
            WorkProgress.IsIndeterminate = Indeterminate;

            if (!Indeterminate && WorkProgress.Value < 0)
            {
                WorkProgress.Value = 0;
            }
        }

        private void Populate()
        {
            ResolutionCombo.Items.Clear();
            ResolutionCombo.Items.Add(Item(string.Format(SRER.GetValue("Portal", "LockScreenVideo", "Original"), VideoWidth, VideoHeight), VideoWidth));
            SelectedWidth = VideoWidth;

            // Fixed steps below the source width, like the reference editor.
            foreach (int Width in new[] { 2560, 1920, 1280, 854 })
            {
                if (VideoWidth > Width)
                {
                    ResolutionCombo.Items.Add(Item($"{Width}px", Width));
                }
            }

            // Connected displays narrower than the source, widest first.
            List<(int Width, int Height)> Displays = GetDisplaySizes();
            int MaxDisplay = 0;

            foreach ((int Width, int Height) in Displays.Distinct().OrderByDescending(Size => Size.Width))
            {
                if (Width <= 0 || Width >= VideoWidth)
                {
                    continue;
                }

                ResolutionCombo.Items.Add(Item(string.Format(SRER.GetValue("Portal", "LockScreenVideo", "Display"), Width, Height), Width));
                MaxDisplay = Math.Max(MaxDisplay, Width);
            }

            // Default to the source width, or the widest display when the
            // source is wider than every connected screen.
            if (MaxDisplay > 0 && Displays.Count > 0 && VideoWidth > Displays.Max(Size => Size.Width))
            {
                SelectedWidth = MaxDisplay;
            }

            SelectByTag(ResolutionCombo, SelectedWidth);

            FpsCombo.Items.Clear();
            FpsCombo.Items.Add(
                Item(
                    VideoFps > 0
                        ? string.Format(SRER.GetValue("Portal", "LockScreenVideo", "AllFrames"), VideoFps.ToString("0.##"))
                        : SRER.GetValue("Portal", "LockScreenVideo", "AllFramesUnknown"),
                    0.0));
            SelectedFps = 0;

            foreach (double Target in new[] { 30.0, 20.0, 15.0, 10.0, 5.0 })
            {
                if (VideoFps > Target)
                {
                    FpsCombo.Items.Add(Item(string.Format(SRER.GetValue("Portal", "LockScreenVideo", "FpsTarget"), Target.ToString("0.##")), Target));
                }
            }

            FpsCombo.SelectedIndex = 0;

            DurationCombo.Items.Clear();
            DurationCombo.Items.Add(
                Item(
                    VideoDuration > 0 ? string.Format(SRER.GetValue("Portal", "LockScreenVideo", "FullVideo"), FormatDuration(VideoDuration)) : SRER.GetValue("Portal", "LockScreenVideo", "FullVideoUnknown"),
                    0));
            DurationCombo.Items.Add(Item(string.Format(SRER.GetValue("Portal", "LockScreenVideo", "FirstSeconds"), 10), 10));
            DurationCombo.Items.Add(Item(string.Format(SRER.GetValue("Portal", "LockScreenVideo", "FirstSeconds"), 30), 30));
            DurationCombo.Items.Add(Item(string.Format(SRER.GetValue("Portal", "LockScreenVideo", "FirstSeconds"), 60), 60));

            SelectByTag(DurationCombo, SelectedSeconds);

            UpdateSummary();
        }

        private static List<(int Width, int Height)> GetDisplaySizes()
        {
            List<(int Width, int Height)> Sizes = new();

            foreach (string Resolution in SSSHQ.GetDisplayResolutions())
            {
                string[] Parts = Resolution.Split('_');

                if (Parts.Length == 2 && int.TryParse(Parts[0], out int Width) && int.TryParse(Parts[1], out int Height) && Width > 0 && Height > 0)
                {
                    Sizes.Add((Width, Height));
                }
            }

            return Sizes;
        }

        private static ComboBoxItem Item(string Content, object Tag)
        {
            return new ComboBoxItem { Content = Content, Tag = Tag };
        }

        private static void SelectByTag(ComboBox Combo, object Tag)
        {
            for (int Index = 0; Index < Combo.Items.Count; Index++)
            {
                if (Combo.Items[Index] is ComboBoxItem Entry && Equals(Entry.Tag, Tag))
                {
                    Combo.SelectedIndex = Index;
                    return;
                }
            }

            Combo.SelectedIndex = 0;
        }

        private static string FormatDuration(double Seconds)
        {
            TimeSpan Span = TimeSpan.FromSeconds(Seconds);

            return Span.TotalHours >= 1 ? Span.ToString(@"h\:mm\:ss") : Span.ToString(@"m\:ss");
        }

        private void Option_SelectionChanged(object sender, RoutedEventArgs e)
        {
            UpdateSummary();
        }

        private void UpdateSummary()
        {
            if (ResolutionCombo.SelectedItem is ComboBoxItem Resolution)
            {
                SelectedWidth = Convert.ToInt32(Resolution.Tag);
            }

            if (FpsCombo.SelectedItem is ComboBoxItem Rate)
            {
                SelectedFps = Convert.ToDouble(Rate.Tag);
            }

            if (DurationCombo.SelectedItem is ComboBoxItem Length)
            {
                SelectedSeconds = Convert.ToInt32(Length.Tag);
            }

            UseHardware = HardwareCheck.IsChecked == true;

            if (Summary is null)
            {
                return;
            }

            string RateText =
                SelectedFps > 0
                    ? string.Format(SRER.GetValue("Portal", "LockScreenVideo", "FpsTarget"), SelectedFps.ToString("0.##"))
                    : SRER.GetValue("Portal", "LockScreenVideo", "AllFramesShort");

            string LengthText =
                SelectedSeconds > 0
                    ? string.Format(SRER.GetValue("Portal", "LockScreenVideo", "FirstSeconds"), SelectedSeconds)
                    : SRER.GetValue("Portal", "LockScreenVideo", "FullVideoShort");

            string Output = string.Join(
                " · ",
                new[] { GetOutputSize(), RateText, LengthText, UseHardware ? SRER.GetValue("Portal", "LockScreenVideo", "HardwareShort") : null }.Where(Part => !string.IsNullOrEmpty(Part)));

            Summary.Text = string.Format(SRER.GetValue("Portal", "LockScreenVideo", "Summary"), Output);
        }

        private string GetOutputSize()
        {
            if (SelectedWidth <= 0 || VideoWidth <= 0 || VideoHeight <= 0)
            {
                return string.Empty;
            }

            // ffmpeg keeps the aspect ratio and rounds the height to an even value.
            int Height = (int)Math.Round(VideoHeight * (double)SelectedWidth / VideoWidth);

            if (Height % 2 != 0)
            {
                Height++;
            }

            return $"{SelectedWidth}x{Math.Max(2, Height)}";
        }

        private string BuildMeta()
        {
            List<string> Parts = new();

            if (VideoWidth > 0 && VideoHeight > 0)
            {
                Parts.Add($"{VideoWidth}x{VideoHeight}");
            }

            if (VideoFps > 0)
            {
                Parts.Add($"{VideoFps.ToString("0.##")} fps");
            }

            if (VideoDuration > 0)
            {
                Parts.Add(FormatDuration(VideoDuration));
            }

            return string.Join(" · ", Parts);
        }

        private void CleanupOutput()
        {
            try
            {
                if (!string.IsNullOrEmpty(_outputDirectory) && Directory.Exists(_outputDirectory))
                {
                    Directory.Delete(_outputDirectory, true);
                }
            }
            catch
            {
            }
        }

        public void Dispose()
        {
            try
            {
                _cancellation.Cancel();
            }
            catch
            {
            }

            if (ResultGifPath is null)
            {
                CleanupOutput();
            }

            try
            {
                if (!string.IsNullOrEmpty(_previewDirectory) && Directory.Exists(_previewDirectory))
                {
                    Directory.Delete(_previewDirectory, true);
                }
            }
            catch
            {
            }

            _loader.Dispose();
            _cancellation.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}
