using CommunityToolkit.Mvvm.ComponentModel;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Threading;
using Wpf.Ui.Controls;
using Button = Wpf.Ui.Controls.Button;
using DialogResult = System.Windows.Forms.DialogResult;
using FolderBrowserDialog = System.Windows.Forms.FolderBrowserDialog;
using SaveFileDialog = Microsoft.Win32.SaveFileDialog;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SMV = Sucrose.Memory.Valuable;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHT = Sucrose.Shared.Space.Helper.Temporary;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSZEZ = Sucrose.Shared.Zip.Extension.Zip;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class SystemSettingViewModel : ViewModel, IDisposable
    {
        private string StoreTemporaryPath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store);

        private string SettingTemporaryPath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder);

        private string CacheTemporaryPath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder);

        private string LogTemporaryPath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.LogFolder);

        private DispatcherTimer InitializeTimer = new();

        private TextBlock LibraryTemporaryHint = new();

        private TextBlock StoreTemporaryHint = new();

        private TextBlock CacheTemporaryHint = new();

        private TextBlock LogTemporaryHint = new();

        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public SystemSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        private void InitializeViewModel()
        {
            TextBlock LogArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Log"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(LogArea);

            SPVCEC LogTemporary = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true
            };

            LogTemporary.LeftIcon.Symbol = SymbolRegular.PersonNote24;
            LogTemporary.Title.Text = SRER.GetValue("Portal", "SystemSettingPage", "LogTemporary");
            LogTemporary.Description.Text = SRER.GetValue("Portal", "SystemSettingPage", "LogTemporary", "Description");

            StackPanel LogTemporaryContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            Button LogTemporaryCreate = new()
            {
                Content = SRER.GetValue("Portal", "SystemSettingPage", "LogTemporary", "Create"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand
            };

            LogTemporaryCreate.Click += (s, e) => LogTemporaryCreateClick(LogTemporaryCreate);

            Button LogTemporaryDelete = new()
            {
                Content = SRER.GetValue("Portal", "SystemSettingPage", "LogTemporary", "Delete"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(10, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand
            };

            LogTemporaryDelete.Click += (s, e) => LogTemporaryDeleteClick(LogTemporaryDelete);

            LogTemporaryContent.Children.Add(LogTemporaryCreate);
            LogTemporaryContent.Children.Add(LogTemporaryDelete);

            LogTemporary.HeaderFrame = LogTemporaryContent;

            LogTemporaryHint = new()
            {
                Text = string.Format(SRER.GetValue("Portal", "SystemSettingPage", "LogTemporary", "Hint"), "0 b"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            LogTemporary.FooterCard = LogTemporaryHint;

            Contents.Add(LogTemporary);

            TextBlock CacheArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Cache"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(CacheArea);

            SPVCEC CacheTemporary = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true
            };

            CacheTemporary.LeftIcon.Symbol = SymbolRegular.FolderBriefcase20;
            CacheTemporary.Title.Text = SRER.GetValue("Portal", "SystemSettingPage", "CacheTemporary");
            CacheTemporary.Description.Text = SRER.GetValue("Portal", "SystemSettingPage", "CacheTemporary", "Description");

            Button CacheTemporaryStart = new()
            {
                Content = SRER.GetValue("Portal", "SystemSettingPage", "CacheTemporary", "Start"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand
            };

            CacheTemporaryStart.Click += (s, e) => CacheTemporaryStartClick(CacheTemporaryStart);

            CacheTemporary.HeaderFrame = CacheTemporaryStart;

            CacheTemporaryHint = new()
            {
                Text = string.Format(SRER.GetValue("Portal", "SystemSettingPage", "CacheTemporary", "Hint"), "0 b"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            CacheTemporary.FooterCard = CacheTemporaryHint;

            Contents.Add(CacheTemporary);

            SPVCEC StoreTemporary = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true
            };

            StoreTemporary.LeftIcon.Symbol = SymbolRegular.FolderGlobe20;
            StoreTemporary.Title.Text = SRER.GetValue("Portal", "SystemSettingPage", "StoreTemporary");
            StoreTemporary.Description.Text = SRER.GetValue("Portal", "SystemSettingPage", "StoreTemporary", "Description");

            Button StoreTemporaryStart = new()
            {
                Content = SRER.GetValue("Portal", "SystemSettingPage", "StoreTemporary", "Start"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand
            };

            StoreTemporaryStart.Click += (s, e) => StoreTemporaryStartClick(StoreTemporaryStart);

            StoreTemporary.HeaderFrame = StoreTemporaryStart;

            StoreTemporaryHint = new()
            {
                Text = string.Format(SRER.GetValue("Portal", "SystemSettingPage", "StoreTemporary", "Hint"), "0 b"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            StoreTemporary.FooterCard = StoreTemporaryHint;

            Contents.Add(StoreTemporary);

            TextBlock LibraryArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Library"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(LibraryArea);

            SPVCEC LibraryTemporary = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true
            };

            LibraryTemporary.LeftIcon.Symbol = SymbolRegular.FolderLink24;
            LibraryTemporary.Title.Text = SRER.GetValue("Portal", "SystemSettingPage", "LibraryTemporary");
            LibraryTemporary.Description.Text = SRER.GetValue("Portal", "SystemSettingPage", "LibraryTemporary", "Description");

            Button LibraryTemporaryStart = new()
            {
                Content = SRER.GetValue("Portal", "SystemSettingPage", "LibraryTemporary", "Start"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand
            };

            LibraryTemporaryStart.Click += (s, e) => LibraryTemporaryStartClick(LibraryTemporaryStart);

            LibraryTemporary.HeaderFrame = LibraryTemporaryStart;

            LibraryTemporaryHint = new()
            {
                Text = string.Format(SRER.GetValue("Portal", "SystemSettingPage", "LibraryTemporary", "Hint"), "0 b"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            LibraryTemporary.FooterCard = LibraryTemporaryHint;

            Contents.Add(LibraryTemporary);

            TextBlock BackupArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Backup"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(BackupArea);

            SPVCEC SettingBackup = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true
            };

            SettingBackup.LeftIcon.Symbol = SymbolRegular.PeopleSettings24;
            SettingBackup.Title.Text = SRER.GetValue("Portal", "SystemSettingPage", "SettingBackup");
            SettingBackup.Description.Text = SRER.GetValue("Portal", "SystemSettingPage", "SettingBackup", "Description");

            StackPanel SettingBackupContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            Button SettingBackupExport = new()
            {
                Content = SRER.GetValue("Portal", "SystemSettingPage", "SettingBackup", "Export"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand
            };

            SettingBackupExport.Click += (s, e) => SettingBackupExportClick(SettingBackupExport);

            Button SettingBackupImport = new()
            {
                Content = SRER.GetValue("Portal", "SystemSettingPage", "SettingBackup", "Import"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(10, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand
            };

            SettingBackupImport.Click += (s, e) => SettingBackupImportClick(SettingBackupImport);

            SettingBackupContent.Children.Add(SettingBackupExport);
            SettingBackupContent.Children.Add(SettingBackupImport);

            SettingBackup.HeaderFrame = SettingBackupContent;

            TextBlock SettingBackupHint = new()
            {
                Text = SRER.GetValue("Portal", "SystemSettingPage", "SettingBackup", "Hint"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            SettingBackup.FooterCard = SettingBackupHint;

            Contents.Add(SettingBackup);

            TextBlock ResetArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Reset"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(ResetArea);

            SPVCEC SettingReset = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true
            };

            SettingReset.LeftIcon.Symbol = SymbolRegular.LauncherSettings24;
            SettingReset.Title.Text = SRER.GetValue("Portal", "SystemSettingPage", "SettingReset");
            SettingReset.Description.Text = SRER.GetValue("Portal", "SystemSettingPage", "SettingReset", "Description");

            Button SettingResetStart = new()
            {
                Content = SRER.GetValue("Portal", "SystemSettingPage", "SettingReset", "Start"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand
            };

            SettingResetStart.Click += (s, e) => SettingResetStartClick(SettingResetStart);

            SettingReset.HeaderFrame = SettingResetStart;

            TextBlock SettingResetHint = new()
            {
                Text = SRER.GetValue("Portal", "SystemSettingPage", "SettingReset", "Hint"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            SettingReset.FooterCard = SettingResetHint;

            Contents.Add(SettingReset);

            InitializeTimer.Tick += new EventHandler(InitializeTimer_Tick);
            InitializeTimer.Interval = new TimeSpan(0, 0, 3);
            InitializeTimer.Start();

            _isInitialized = true;
        }

        public override void OnNavigatedFrom()
        {
            InitializeTimer.Stop();
        }

        private async Task CreateLog()
        {
            SaveFileDialog SaveDialog = new()
            {
                FileName = SMV.LogCompress,

                Filter = SRER.GetValue("Portal", "SystemSettingPage", "Log", "SaveDialogFilter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Portal", "SystemSettingPage", "Log", "SaveDialogTitle"),

                InitialDirectory = SMR.DesktopPath
            };

            if (SaveDialog.ShowDialog() == true)
            {
                string Destination = SaveDialog.FileName;

                string[] Sources = new[]
                {
                    LogTemporaryPath,
                    SettingTemporaryPath,
                };

                string[] Excludes = new[]
                {
                    SMMI.PrivateSettingManager.SettingFile()
                };

                SSZEZ.Compress(Sources, Excludes, Destination);
            }

            await Task.CompletedTask;
        }

        private void SettingResetStartClick(Button SettingResetStart)
        {
            SettingResetStart.IsEnabled = false;

            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Reset}{SMR.ValueSeparator}{SSSMI.Launcher}");
        }

        private void LogTemporaryDeleteClick(Button LogTemporaryDelete)
        {
            LogTemporaryDelete.IsEnabled = false;

            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Temp}{SMR.ValueSeparator}{LogTemporaryPath}{SMR.ValueSeparator}{SSSMI.Launcher}");
        }

        private void CacheTemporaryStartClick(Button CacheTemporaryStart)
        {
            CacheTemporaryStart.IsEnabled = false;

            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Temp}{SMR.ValueSeparator}{CacheTemporaryPath}{SMR.ValueSeparator}{SSSMI.Launcher}");
        }

        private void StoreTemporaryStartClick(Button StoreTemporaryStart)
        {
            StoreTemporaryStart.IsEnabled = false;

            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Temp}{SMR.ValueSeparator}{StoreTemporaryPath}{SMR.ValueSeparator}{SSSMI.Launcher}");
        }

        private void SettingBackupExportClick(Button SettingBackupExport)
        {
            SettingBackupExport.IsEnabled = false;

            FolderBrowserDialog BrowserDialog = new()
            {
                ShowNewFolderButton = true
            };

            if (BrowserDialog.ShowDialog() == DialogResult.OK && !string.IsNullOrEmpty(BrowserDialog.SelectedPath))
            {
                string Destination = BrowserDialog.SelectedPath;

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Export}{SMR.ValueSeparator}{Destination}{SMR.ValueSeparator}{SSSMI.Launcher}");
            }
            else
            {
                SettingBackupExport.IsEnabled = true;
            }
        }

        private void SettingBackupImportClick(Button SettingBackupImport)
        {
            SettingBackupImport.IsEnabled = false;

            FolderBrowserDialog BrowserDialog = new()
            {
                ShowNewFolderButton = true
            };

            if (BrowserDialog.ShowDialog() == DialogResult.OK && !string.IsNullOrEmpty(BrowserDialog.SelectedPath))
            {
                string Destination = BrowserDialog.SelectedPath;

                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Import}{SMR.ValueSeparator}{Destination}{SMR.ValueSeparator}{SSSMI.Launcher}");
            }
            else
            {
                SettingBackupImport.IsEnabled = true;
            }
        }

        private void LibraryTemporaryStartClick(Button LibraryTemporaryStart)
        {
            LibraryTemporaryStart.IsEnabled = false;

            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Temp}{SMR.ValueSeparator}{SMMM.LibraryLocation}{SMR.ValueSeparator}{SSSMI.Launcher}");
        }

        private async void LogTemporaryCreateClick(Button LogTemporaryCreate)
        {
            LogTemporaryCreate.IsEnabled = false;

            await Task.Run(CreateLog);

            LogTemporaryCreate.IsEnabled = true;
        }

        private async void InitializeTimer_Tick(object sender, EventArgs e)
        {
            try
            {
                string LogTemporarySize = await SSSHT.Size(LogTemporaryPath);

                LogTemporaryHint.Text = string.Format(SRER.GetValue("Portal", "SystemSettingPage", "LogTemporary", "Hint"), LogTemporarySize);

                string CacheTemporarySize = await SSSHT.Size(CacheTemporaryPath);

                CacheTemporaryHint.Text = string.Format(SRER.GetValue("Portal", "SystemSettingPage", "CacheTemporary", "Hint"), CacheTemporarySize);

                string StoreTemporarySize = await SSSHT.Size(StoreTemporaryPath);

                StoreTemporaryHint.Text = string.Format(SRER.GetValue("Portal", "SystemSettingPage", "StoreTemporary", "Hint"), StoreTemporarySize);

                string LibraryTemporarySize = await SSSHT.Size(SMMM.LibraryLocation);

                LibraryTemporaryHint.Text = string.Format(SRER.GetValue("Portal", "SystemSettingPage", "LibraryTemporary", "Hint"), LibraryTemporarySize);
            }
            catch { }
        }

        public void Dispose()
        {
            Contents.Clear();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}