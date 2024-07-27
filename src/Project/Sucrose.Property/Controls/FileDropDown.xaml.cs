using System.IO;
using MessageBox = Wpf.Ui.Controls.MessageBox;
using OpenFileDialog = Microsoft.Win32.OpenFileDialog;
using SMR = Sucrose.Memory.Readonly;
using SPHL = Sucrose.Property.Helper.Localization;
using SPHP = Sucrose.Property.Helper.Properties;
using SPMI = Sucrose.Property.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSSHA = Sucrose.Shared.Space.Helper.Access;
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
            Data.Text = SPHL.Convert(Data.Text);
            Data.Value = SPHL.Convert(Data.Value);

            Component_Items(Data);
            Label.Text = Data.Text;
            Component.Text = Data.Value;

            Command.Click += async (s, e) => await Command_Click(Data);

            Component.SelectionChanged += (s, e) => Component_Changed(Key, Data, $"{Component.SelectedValue}");

            if (!string.IsNullOrEmpty(Data.Help))
            {
                Data.Help = SPHL.Convert(Data.Help);

                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Component.ToolTip = HelpTip;
            }

            ToolTip CommandTip = new()
            {
                Content = SRER.GetValue("Property", "FileDropDown", "Tip")
            };

            Command.ToolTip = CommandTip;
        }

        private void Component_Items(SSTMFDDM Data)
        {
            string Folder = Path.Combine(SPMI.Path, Data.Folder);

            if (Directory.Exists(Folder))
            {
                string[] Extensions = Data.Filter.Replace("*", "").Split('|');
                string[] Files = Directory.GetFiles(Folder, "*.*", SearchOption.TopDirectoryOnly).Where(Record => Extensions.Any(Extension => Record.EndsWith(Extension))).ToArray();

                foreach (string File in Files)
                {
                    Component.Items.Add(Path.GetFileName(File));
                }

                Component.SelectedValue = Data.Value;
            }
        }

        private async Task Command_Click(SSTMFDDM Data)
        {
            string Filter = $"{Data.Desc} ({Data.Filter.Replace("|", ", ")})|{Data.Filter.Replace('|', ';')}";

            Filter += $"|{SRER.GetValue("Property", "FileDropDown", "Filter")}";

            OpenFileDialog FileDialog = new()
            {
                Filter = Filter,
                FilterIndex = 1,

                Title = Data.Title,

                Multiselect = false,

                InitialDirectory = SMR.DesktopPath
            };

            if (FileDialog.ShowDialog() == true)
            {
                if (SSSHA.File(FileDialog.FileName))
                {
                    string FileName = Path.GetFileName(FileDialog.FileName);

                    using (FileStream Source = new(FileDialog.FileName, FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
                    {
                        string Target = Path.Combine(SPMI.Path, Data.Folder, FileName);

                        if (File.Exists(Target))
                        {
                            File.Delete(Target);
                        }

                        using FileStream Destination = new(Target, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.ReadWrite);

                        Source.CopyTo(Destination);
                    }

                    await Task.Delay(500);

                    if (!Component.Items.OfType<string>().Any(Item => Item == FileName))
                    {
                        Component.Items.Add(FileName);
                    }

                    Component.SelectedValue = FileName;
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

            await Task.CompletedTask;
        }

        private void Component_Changed(string Key, SSTMFDDM Data, string Value)
        {
            Data.Value = Value;

            SPHP.Change(Key, Data);
        }
    }
}