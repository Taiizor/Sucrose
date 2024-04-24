using System.IO;
using OpenFileDialog = Microsoft.Win32.OpenFileDialog;
using SMR = Sucrose.Memory.Readonly;
using SPHP = Sucrose.Property.Helper.Properties;
using SPMI = Sucrose.Property.Manage.Internal;
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
            Label.Text = Data.Text;
            Component.Text = Data.Value;

            Command.Click += async (s, e) => await Command_Click(Data);

            Component.SelectionChanged += (s, e) => Component_Changed(Key, Data, $"{Component.SelectedValue}");

            if (!string.IsNullOrEmpty(Data.Help))
            {
                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Container.ToolTip = HelpTip;
            }
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

            Filter += "|All Files (*.*)|*.*";

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
                string FileName = Path.GetFileName(FileDialog.FileName);

                using (FileStream Source = new(FileDialog.FileName, FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
                {
                    using FileStream Destination = new(Path.Combine(SPMI.Path, Data.Folder, FileName), FileMode.Create, FileAccess.Write);

                    Source.CopyTo(Destination);
                }

                await Task.Delay(500);

                Component.Items.Add(FileName);

                Component.SelectedValue = FileName;
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