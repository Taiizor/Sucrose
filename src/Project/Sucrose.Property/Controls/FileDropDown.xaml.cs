using System.IO;
using MessageBox = Wpf.Ui.Controls.MessageBox;
using OpenFileDialog = Microsoft.Win32.OpenFileDialog;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SPHL = Sucrose.Property.Helper.Localization;
using SPHP = Sucrose.Property.Helper.Properties;
using SPMI = Sucrose.Property.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSHL = Sucrose.Shared.Space.Helper.Lock;
using SSTMFDDM = Sucrose.Shared.Theme.Model.FileDropDownModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// FileDropDown.xaml etkileşim mantığı
    /// </summary>
    public partial class FileDropDown : UserControl
    {
        public FileDropDown(string Key, SSTMFDDM Data)
        {
            InitializeComponent();

            InitializeData(Key, Data);
        }

        private void InitializeData(string Key, SSTMFDDM Data)
        {
            Component_Items(Data);

            Label.Text = SPHL.Convert(Data.Text);
            Component.Text = SPHL.Convert(Data.Value);

            Open.Click += async (s, e) => await Open_Click(Data);
            Delete.Click += async (s, e) => await Delete_Click(Data);

            Component.SelectionChanged += (s, e) => Component_Changed(Key, Data, $"{Component.SelectedValue}");

            if (!string.IsNullOrEmpty(Data.Help))
            {
                ToolTip HelpTip = new()
                {
                    Content = SPHL.Convert(Data.Help)
                };

                Component.ToolTip = HelpTip;
            }

            ToolTip OpenTip = new()
            {
                Content = SRER.GetValue("Property", "FileDropDown", "OpenTip")
            };

            Open.ToolTip = OpenTip;

            ToolTip DeleteTip = new()
            {
                Content = SRER.GetValue("Property", "FileDropDown", "DeleteTip")
            };

            Delete.ToolTip = DeleteTip;
        }

        private void Component_Items(SSTMFDDM Data)
        {
            string Folder = Path.Combine(SPMI.Path, SPHL.Convert(Data.Folder));

            if (Directory.Exists(Folder))
            {
                string[] Extensions = SPHL.Convert(Data.Filter).Replace("*", "").Split('|');
                string[] Files = Directory.GetFiles(Folder, "*.*", SearchOption.TopDirectoryOnly).Where(Record => Extensions.Any(Extension => !string.IsNullOrEmpty(Extension) && Record.EndsWith(Extension, StringComparison.OrdinalIgnoreCase))).ToArray();

                foreach (string File in Files)
                {
                    Component.Items.Add(Path.GetFileName(File));
                }

                if (Files.Count() > 1)
                {
                    Delete.IsEnabled = true;
                }

                Component.SelectedValue = SPHL.Convert(Data.Value);
            }
        }

        private async Task Open_Click(SSTMFDDM Data)
        {
            if (Directory.Exists(SPMI.Path))
            {
                string Filter = $"{SPHL.Convert(Data.Desc)} ({SPHL.Convert(Data.Filter).Replace("|", ", ")})|{SPHL.Convert(Data.Filter).Replace('|', ';')}";

                Filter += $"|{SRER.GetValue("Property", "FileDropDown", "Filter")}";

                OpenFileDialog FileDialog = new()
                {
                    Filter = Filter,
                    FilterIndex = 1,

                    Multiselect = false,

                    InitialDirectory = SMMRP.Desktop,

                    Title = SPHL.Convert(Data.Title)
                };

                if (FileDialog.ShowDialog() == true)
                {
                    if (SSSHA.File(FileDialog.FileName))
                    {
                        if (SSSHL.File(FileDialog.FileName))
                        {
                            string FileName = Path.GetFileName(FileDialog.FileName);
                            string Target = Path.Combine(SPMI.Path, SPHL.Convert(Data.Folder), FileName);

                            // A file picked from the wallpaper's own folder is already in place; deleting the target would delete the source
                            if (!string.Equals(Path.GetFullPath(FileDialog.FileName), Path.GetFullPath(Target), StringComparison.OrdinalIgnoreCase))
                            {
                                if (File.Exists(Target))
                                {
                                    await Task.Run(() => SSSHF.Delete(Target));
                                }

                                await Task.Run(() => SSSHF.CopyBuffer(FileDialog.FileName, Target));
                            }

                            await Task.Delay(500);

                            if (!Component.Items.OfType<string>().Any(Item => Item == FileName))
                            {
                                Component.Items.Add(FileName);
                            }

                            Component.SelectedValue = FileName;

                            if (Component.Items.OfType<string>().Count() > 1)
                            {
                                Delete.IsEnabled = true;
                            }
                        }
                        else
                        {
                            MessageBox Warning = new()
                            {
                                Title = SRER.GetValue("Property", "FileDropDown", "Lock", "Title"),
                                Content = SRER.GetValue("Property", "FileDropDown", "Lock", "Message"),
                                CloseButtonText = SRER.GetValue("Property", "FileDropDown", "Lock", "Close")
                            };

                            await Warning.ShowDialogAsync();
                        }
                    }
                    else
                    {
                        MessageBox Warning = new()
                        {
                            Title = SRER.GetValue("Property", "FileDropDown", "Access", "Title"),
                            Content = SRER.GetValue("Property", "FileDropDown", "Access", "Message"),
                            CloseButtonText = SRER.GetValue("Property", "FileDropDown", "Access", "Close")
                        };

                        await Warning.ShowDialogAsync();
                    }
                }
            }

            await Task.CompletedTask;
        }

        private async Task Delete_Click(SSTMFDDM Data)
        {
            Delete.IsEnabled = false;

            if (Directory.Exists(SPMI.Path))
            {
                if (Component.SelectedValue != null)
                {
                    int Index = Component.SelectedIndex;
                    string FileName = $"{Component.SelectedValue}";
                    string Target = Path.Combine(SPMI.Path, SPHL.Convert(Data.Folder), FileName);

                    Component.Items.Remove(FileName);

                    if (File.Exists(Target))
                    {
                        await Task.Run(() => SSSHF.Delete(Target));
                    }

                    if (Index > 0)
                    {
                        Component.SelectedIndex = --Index;
                    }
                    else
                    {
                        Component.SelectedIndex = 0;
                    }
                }
            }

            if (Component.Items.OfType<string>().Count() > 1)
            {
                Delete.IsEnabled = true;
            }

            await Task.CompletedTask;
        }

        private void Component_Changed(string Key, SSTMFDDM Data, string Value)
        {
            Data.Value = Value;

            SPHP.Change(Key, Data);
        }
    }
}