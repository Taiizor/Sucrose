using System.IO;
using System.Windows;
using Wpf.Ui.Abstractions.Controls;
using SHG = Skylark.Helper.Generator;
using SMMCW = Sucrose.Memory.Manage.Constant.Warehouse;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMML = Sucrose.Manager.Manage.Library;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMW = Sucrose.Manager.Manage.Warehouse;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCTI = Sucrose.Portal.Views.Controls.ThemeImport;
using SPVMPLVM = Sucrose.Portal.ViewModels.Pages.LibraryViewModel;
using SPVPLELP = Sucrose.Portal.Views.Pages.Library.EmptyLibraryPage;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SPVPLFLP = Sucrose.Portal.Views.Pages.Library.FullLibraryPage;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDESKT = Sucrose.Shared.Dependency.Enum.SortKindType;
using SSDESMT = Sucrose.Shared.Dependency.Enum.SortModeType;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSSHC = Sucrose.Shared.Space.Helper.Copy;
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
            string ShowcasePath = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMR.Showcase);

            if (Directory.Exists(ShowcasePath))
            {
                bool State = false;

                List<string> Showcase = SMMW.Showcase;

                if (!Directory.Exists(SMML.LibraryLocation))
                {
                    Directory.CreateDirectory(SMML.LibraryLocation);
                }

                foreach (string Directory in Directory.GetDirectories(ShowcasePath, "*", SearchOption.TopDirectoryOnly).Where(Directory => !Showcase.Contains(Path.GetFileName(Directory))))
                {
                    SSSHC.Folder(Directory, Path.Combine(SMML.LibraryLocation, Path.GetFileName(Directory)), false);

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

            if (Directory.Exists(SMML.LibraryLocation))
            {
                string[] Folders = Directory.GetDirectories(SMML.LibraryLocation);

                if (Folders != null && Folders.Any())
                {
                    foreach (string Folder in Folders)
                    {
                        string PropertiesCache = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.Properties);
                        string PropertiesFile = Path.Combine(PropertiesCache, $"{Path.GetFileName(Folder)}.json");
                        string InfoPath = Path.Combine(Folder, SMR.SucroseInfo);

                        if (File.Exists(InfoPath))
                        {
                            string InfoContent = SSTHI.ReadInfo(InfoPath);

                            if (InfoContent != null && SSTHI.CheckJson(InfoContent))
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

                        if (SMML.LibraryDelete && SSSHA.Directory(Folder))
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
                    Dictionary<string, object> SortThemes = new();

                    foreach (string Theme in Themes.ToList())
                    {
                        string ThemePath = Path.Combine(SMML.LibraryLocation, Theme);
                        string InfoPath = Path.Combine(ThemePath, SMR.SucroseInfo);

                        if (Directory.Exists(ThemePath) && File.Exists(InfoPath))
                        {
                            SSTHI Info = SSTHI.ReadJson(InfoPath);

                            IEnumerable<string> SearchText = Info.Title.Split(' ')
                                .Concat(Info.Description.Split(' '))
                                .Concat(Info.Tags?.SelectMany(Tag => Tag.Split(' ')) ?? Array.Empty<string>());

                            Searches.Add(Theme, string.Join(" ", SearchText.Select(Word => Word.ToLowerInvariant()).Distinct()));

                            if (SSDMMP.LibrarySortMode == SSDESMT.Name)
                            {
                                SortThemes.Add(Theme, Info.Title);
                            }
                            else if (SSDMMP.LibrarySortMode == SSDESMT.Creation)
                            {
                                SortThemes.Add(Theme, Directory.GetCreationTime(Path.Combine(SMML.LibraryLocation, Theme)));
                            }
                            else if (SSDMMP.LibrarySortMode == SSDESMT.Modification)
                            {
                                SortThemes.Add(Theme, File.GetLastWriteTime(InfoPath));
                            }
                        }
                    }

                    if (SortThemes != null && SortThemes.Any())
                    {
                        if (SSDMMP.LibrarySortKind == SSDESKT.Ascending)
                        {
                            SortThemes = SortThemes.OrderBy(Theme => Theme.Value).ToDictionary(Theme => Theme.Key, Theme => Theme.Value);
                        }
                        else
                        {
                            SortThemes = SortThemes.OrderByDescending(Theme => Theme.Value).ToDictionary(Theme => Theme.Key, Theme => Theme.Value);
                        }

                        Themes.Clear();
                        Themes.AddRange(SortThemes.Select(Theme => Theme.Key));
                    }
                }
            }
            else
            {
                if (Themes != null && Themes.Any())
                {
                    Themes.Clear();
                }

                Directory.CreateDirectory(SMML.LibraryLocation);
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
                            if (!Directory.Exists(SMML.LibraryLocation))
                            {
                                Directory.CreateDirectory(SMML.LibraryLocation);
                            }

                            string Name;

                            do
                            {
                                Name = SHG.GenerateString(SMMM.Chars, 25, SMMRG.Randomise);
                            } while (Directory.Exists(Path.Combine(SMML.LibraryLocation, Name)));

                            Result = await Task.Run(() => SSZEZ.Extract(Record, Path.Combine(SMML.LibraryLocation, Name)));

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