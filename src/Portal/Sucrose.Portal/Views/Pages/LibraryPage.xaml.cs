using System.IO;
using System.Windows;
using Wpf.Ui.Abstractions.Controls;
using SHG = Skylark.Helper.Generator;
using SMMCW = Sucrose.Memory.Manage.Constant.Warehouse;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMML = Sucrose.Manager.Manage.Library;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMVA = Sucrose.Memory.Manage.Valuable.App;
using SMMW = Sucrose.Manager.Manage.Warehouse;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCTI = Sucrose.Portal.Views.Controls.ThemeImport;
using SPVMPLVM = Sucrose.Portal.ViewModels.Pages.LibraryViewModel;
using SPVPLELP = Sucrose.Portal.Views.Pages.Library.EmptyLibraryPage;
using SPVPLFLP = Sucrose.Portal.Views.Pages.Library.FullLibraryPage;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
using SSSHC = Sucrose.Shared.Space.Helper.Copy;
using SSSHS = Sucrose.Shared.Space.Helper.Sort;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSZEZ = Sucrose.Shared.Zip.Extension.Zip;
using SSZHA = Sucrose.Shared.Zip.Helper.Archive;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// LibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class LibraryPage : INavigableView<SPVMPLVM>, IDisposable
    {
        private Dictionary<string, string> Searches = new();

        private SPVPLELP EmptyLibraryPage { get; set; }

        private SPVPLFLP FullLibraryPage { get; set; }

        private static List<string> Themes = new();

        public SPVMPLVM ViewModel { get; }

        public LibraryPage(SPVMPLVM ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

            CheckShowcase();

            InitializeThemes();

            InitializeComponent();

            Library();

            Search();
        }

        private void Search()
        {
            string Search = SPMI.SearchService.SearchText;

            SPMI.SearchService.Dispose();

            SPMI.SearchService = new()
            {
                SearchText = Search
            };

            SPMI.SearchService.SearchTextChanged += SearchService_SearchTextChanged;
        }

        private void Library()
        {
            SPMI.LibraryService.Dispose();

            SPMI.LibraryService = new();

            SPMI.LibraryService.CreatedWallpaper += LibraryService_CreatedWallpaper;
        }

        private void CheckShowcase()
        {
            string ShowcasePath = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Showcase);

            if (Directory.Exists(ShowcasePath))
            {
                bool State = false;

                List<string> Showcase = SMMW.Showcase;

                if (!Directory.Exists(SMML.Location))
                {
                    Directory.CreateDirectory(SMML.Location);
                }

                foreach (string Directory in Directory.GetDirectories(ShowcasePath, "*", SearchOption.TopDirectoryOnly).Where(Directory => !Showcase.Contains(Path.GetFileName(Directory))))
                {
                    SSSHC.Folder(Directory, Path.Combine(SMML.Location, Path.GetFileName(Directory)), false);

                    Showcase.Add(Path.GetFileName(Directory));

                    State = true;
                }

                if (State)
                {
                    SMMI.WarehouseSettingManager.SetSetting(SMMCW.Showcase, Showcase);
                }
            }
        }

        private void InitializeThemes()
        {
            if (Searches != null && Searches.Any())
            {
                Searches.Clear();
            }

            if (Directory.Exists(SMML.Location))
            {
                string[] Folders = Directory.GetDirectories(SMML.Location);

                if (Folders != null && Folders.Any())
                {
                    foreach (string Folder in Folders)
                    {
                        string PropertiesCache = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.Properties);
                        string PropertiesFile = Path.Combine(PropertiesCache, $"{Path.GetFileName(Folder)}.json");
                        string InfoPath = Path.Combine(Folder, SMMRC.SucroseInfo);

                        if (File.Exists(InfoPath))
                        {
                            string InfoContent = SSTHI.Read(InfoPath);

                            if (InfoContent != null && SSTHI.FromCheck(InfoContent))
                            {
                                SSTHI Info = SSTHI.FromJson(InfoContent);

                                if (Info != null && !string.IsNullOrEmpty(Info.Preview) && !string.IsNullOrEmpty(Info.Thumbnail))
                                {
                                    string PreviewPath = Path.Combine(Folder, Info.Preview);
                                    string ThumbnailPath = Path.Combine(Folder, Info.Thumbnail);

                                    if (File.Exists(PreviewPath) && File.Exists(ThumbnailPath) && (SSTHV.IsUrl(Info.Source) || File.Exists(Path.Combine(Folder, Info.Source))))
                                    {
                                        if (!Themes.Contains(Path.GetFileName(Folder)))
                                        {
                                            Themes.Add(Path.GetFileName(Folder));
                                        }

                                        continue;
                                    }
                                }
                            }
                        }

                        if (SMML.DeleteCorrupt && SSSHA.Directory(Folder))
                        {
                            Directory.Delete(Folder, true);

                            if (File.Exists(PropertiesFile))
                            {
                                File.Delete(PropertiesFile);
                            }
                        }
                    }
                }

                if (Themes != null && Themes.Any())
                {
                    (Themes, Searches) = SSSHS.Theme(Themes, Searches);
                }
            }
            else
            {
                if (Themes != null && Themes.Any())
                {
                    Themes.Clear();
                }

                Directory.CreateDirectory(SMML.Location);
            }
        }

        private async Task Start(bool Progress = false)
        {
            if (Themes != null && Themes.Any())
            {
                FullLibraryPage = new(Searches, Themes);

                FrameLibrary.Content = FullLibraryPage;
            }
            else
            {
                EmptyLibraryPage = new();

                FrameLibrary.Content = EmptyLibraryPage;
            }

            if (!Progress)
            {
                await Task.Delay(500);

                FrameLibrary.Visibility = Visibility.Visible;
                ProgressLibrary.Visibility = Visibility.Collapsed;
            }
        }

        private void GridLibrary_DragOver(object sender, DragEventArgs e)
        {
            if (!e.Data.GetDataPresent(DataFormats.FileDrop) || e.AllowedEffects.HasFlag(DragDropEffects.Copy) == false)
            {
                e.Effects = DragDropEffects.None;
            }
            else
            {
                e.Effects = DragDropEffects.Copy;
                DropRectangle.Visibility = Visibility.Visible;
            }
        }

        private void GridLibrary_DragLeave(object sender, DragEventArgs e)
        {
            DropRectangle.Visibility = Visibility.Collapsed;
        }

        private async void GridLibrary_Drop(object sender, DragEventArgs e)
        {
            DropRectangle.Visibility = Visibility.Collapsed;

            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                bool State = false;
                List<SSDECT> Types = new();
                List<string> Messages = new();

                string[] Files = (string[])e.Data.GetData(DataFormats.FileDrop);

                if (Files.Any())
                {
                    foreach (string Record in Files)
                    {
                        SSDECT Result = SSZHA.Check(Record);

                        if (Result == SSDECT.Pass)
                        {
                            if (!Directory.Exists(SMML.Location))
                            {
                                Directory.CreateDirectory(SMML.Location);
                            }

                            string Name;

                            do
                            {
                                Name = SHG.GenerateString(SMMVA.Chars, 25, SMMRG.Randomise);
                            } while (Directory.Exists(Path.Combine(SMML.Location, Name)));

                            Result = await Task.Run(() => SSZEZ.Extract(Record, Path.Combine(SMML.Location, Name)));

                            if (Result == SSDECT.Pass)
                            {
                                State = true;
                                Messages.Add(string.Format(SRER.GetValue("Portal", "LibraryPage", "Import", "Successful"), Path.GetFileNameWithoutExtension(Record)));
                            }
                            else
                            {
                                Messages.Add(string.Format(SRER.GetValue("Portal", "LibraryPage", "Import", "Unsuccessful"), Path.GetFileNameWithoutExtension(Record), Result));
                            }
                        }
                        else
                        {
                            if (!Types.Contains(Result))
                            {
                                Types.Add(Result);
                            }

                            Messages.Add(string.Format(SRER.GetValue("Portal", "LibraryPage", "Import", "Unsuccessful"), Path.GetFileNameWithoutExtension(Record), Result));
                        }
                    }
                }

                if (Messages.Any() || Types.Any())
                {
                    SPVCTI ThemeImport = new()
                    {
                        Types = Types,
                        Messages = Messages
                    };

                    await ThemeImport.ShowAsync();

                    ThemeImport.Dispose();
                }

                if (State)
                {
                    Dispose();

                    InitializeThemes();

                    await Start(true);
                }
            }
        }

        private async void GridLibrary_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }

        private async void SearchService_SearchTextChanged(object sender, EventArgs e)
        {
            if (string.IsNullOrEmpty(SPMI.SearchService.SearchText) || !string.IsNullOrWhiteSpace(SPMI.SearchService.SearchText))
            {
                Dispose();

                await Start(true);
            }
        }

        private async void LibraryService_CreatedWallpaper(object sender, EventArgs e)
        {
            Dispose();

            InitializeThemes();

            await Start(true);
        }

        public void Dispose()
        {
            FullLibraryPage?.Dispose();
            EmptyLibraryPage?.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}