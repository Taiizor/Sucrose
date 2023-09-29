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
using SMR = Sucrose.Memory.Readonly;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHT = Sucrose.Shared.Space.Helper.Temporary;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class SystemSettingViewModel : ObservableObject, INavigationAware, IDisposable
    {
        private string StoreTemporaryPath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store);

        private string CacheTemporaryPath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder);

        private DispatcherTimer InitializeTimer = new();

        private TextBlock StoreTemporaryHint = new();

        private TextBlock CacheTemporaryHint = new();

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
            TextBlock CacheArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "Cache"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(CacheArea);

            SPVCEC CacheTemporary = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true,
                IsExpand = true
            };

            CacheTemporary.Title.Text = "Önbellek Temizleme";
            CacheTemporary.LeftIcon.Symbol = SymbolRegular.FolderBriefcase20;
            CacheTemporary.Description.Text = "Uygulamanın oluşturmuş olduğu tüm önbellek dosyalarını temizleme.";

            Button CacheTemporaryStart = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Content = "Temizliğe Başla",
                Padding = new Thickness(8),
                Cursor = Cursors.Hand,
            };

            CacheTemporaryStart.Click += (s, e) => CacheTemporaryStartClick(CacheTemporaryStart);

            CacheTemporary.HeaderFrame = CacheTemporaryStart;

            CacheTemporaryHint = new()
            {
                Text = "Not: Önbellek dosyalarının kapladığı toplam alan 0 b. Temizliğe başlandığında tüm uygulamalar kapanacaktır.",
                Foreground = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            CacheTemporary.FooterCard = CacheTemporaryHint;

            Contents.Add(CacheTemporary);

            SPVCEC StoreTemporary = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true,
                IsExpand = true
            };

            StoreTemporary.Title.Text = "Mağaza Temizleme";
            StoreTemporary.LeftIcon.Symbol = SymbolRegular.FolderGlobe20;
            StoreTemporary.Description.Text = "Uygulamanın oluşturmuş olduğu tüm mağaza dosyalarını temizleme.";

            Button StoreTemporaryStart = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Content = "Temizliğe Başla",
                Padding = new Thickness(8),
                Cursor = Cursors.Hand,
            };

            StoreTemporaryStart.Click += (s, e) => StoreTemporaryStartClick(StoreTemporaryStart);

            StoreTemporary.HeaderFrame = StoreTemporaryStart;

            StoreTemporaryHint = new()
            {
                Text = "Not: Mağaza dosyalarının kapladığı toplam alan 0 b. Temizliğe başlandığında tüm uygulamalar kapanacaktır.",
                Foreground = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            StoreTemporary.FooterCard = StoreTemporaryHint;

            Contents.Add(StoreTemporary);

            TextBlock BackupArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "Backup"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(BackupArea);

            SPVCEC SettingBackup = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true
            };

            SettingBackup.Title.Text = "Ayar Yedekleme";
            SettingBackup.LeftIcon.Symbol = SymbolRegular.PeopleSettings24;
            SettingBackup.Description.Text = "Uygulamanın oluşturmuş olduğu tüm ayar dosyalarını yönetme.";

            StackPanel SettingBackupContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            Button SettingBackupExport = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(0, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand,
                Content = "Yedekle"
            };

            SettingBackupExport.Click += (s, e) => SettingBackupExportClick(SettingBackupExport);

            Button SettingBackupImport = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Secondary,
                Margin = new Thickness(10, 0, 0, 0),
                Padding = new Thickness(8),
                Cursor = Cursors.Hand,
                Content = "Geri Yükle"
            };

            SettingBackupImport.Click += (s, e) => SettingBackupImportClick(SettingBackupImport);

            SettingBackupContent.Children.Add(SettingBackupExport);
            SettingBackupContent.Children.Add(SettingBackupImport);

            SettingBackup.HeaderFrame = SettingBackupContent;

            TextBlock SettingBackupHint = new()
            {
                Text = "Not: Yedekleme işlemi başladığında tüm uygulamalar kapanacaktır.",
                Foreground = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            SettingBackup.FooterCard = SettingBackupHint;

            Contents.Add(SettingBackup);

            InitializeTimer.Tick += new EventHandler(InitializeTimer_Tick);
            InitializeTimer.Interval = new TimeSpan(0, 0, 3);
            InitializeTimer.Start();

            _isInitialized = true;
        }

        public void OnNavigatedTo()
        {
            //
        }

        public void OnNavigatedFrom()
        {
            //Dispose();
            InitializeTimer.Stop();
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

        private async void InitializeTimer_Tick(object sender, EventArgs e)
        {
            try
            {
                string CacheTemporarySize = await SSSHT.Size(CacheTemporaryPath);

                CacheTemporaryHint.Text = $"Not: Önbellek dosyalarının kapladığı toplam alan {CacheTemporarySize}. Temizliğe başlandığında tüm uygulamalar kapanacaktır.";

                string StoreTemporarySize = await SSSHT.Size(StoreTemporaryPath);

                StoreTemporaryHint.Text = $"Not: Mağaza dosyalarının kapladığı toplam alan {StoreTemporarySize}. Temizliğe başlandığında tüm uygulamalar kapanacaktır.";
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