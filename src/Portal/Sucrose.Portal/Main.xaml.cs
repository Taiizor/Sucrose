using System.Windows;
using System.Windows.Controls;
using SSIW = Sucrose.Signal.Interface.Websiter;
using SSMI = Sucrose.Signal.Manage.Internal;

namespace Sucrose.Portal
{
    /// <summary>
    /// Interaction logic for Main.xaml
    /// </summary>
    public partial class Main : Window
    {
        private readonly string Uri1 = "https://www.vegalya.com/3/0wj1biqk.f41/fluid.html";
        private readonly string Uri2 = "https://www.vegalya.com/3/0wj1biqk.f412/fluid.html";
        private readonly string Uri3 = "https://www.vegalya.com/3/iqdvd4pt.jyo/triangle.html";
        private readonly string Uri4 = "https://www.vegalya.com/3/lgz4xpht.sjn/index.html";
        private readonly string Uri5 = "https://www.vegalya.com/3/mrn3a0m5.0lk/index.html";
        private readonly string Uri6 = "https://www.vegalya.com/3/rtnm43pj.wyl/index.html";
        private readonly string Uri7 = "https://www.vegalya.com/3/xn0quq52.bq2/index.html";
        private readonly string Uri8 = "https://www.vegalya.com/3/wxqfmbno.3vk/index.html";
        private readonly string Uri9 = "https://www.vegalya.com/3/ptpcxvcd.hlz/index.html";
        private readonly string Uri10 = "https://www.vegalya.com/3/nps35xrp.5eh/jellyfish.html";

        public Main()
        {
            InitializeComponent();
        }

        private void Button_Click(object sender, RoutedEventArgs e)
        {
            switch ((sender as Button).Content)
            {
                case "Design 1":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri1, Hook = true });
                    break;
                case "Design 2":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri2, Hook = true });
                    break;
                case "Design 3":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri3, Hook = true });
                    break;
                case "Design 4":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri4, Hook = false });
                    break;
                case "Design 5":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri5, Hook = true });
                    break;
                case "Design 6":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri6, Hook = true });
                    break;
                case "Design 7":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri7, Hook = false });
                    break;
                case "Design 8":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri8, Hook = true });
                    break;
                case "Design 9":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri9, Hook = true });
                    break;
                case "Design 10":
                    SSMI.WebsiterManager.FileSave<SSIW>(new() { Url = Uri10, Hook = true });
                    break;
                default:
                    break;
            }
        }
    }
}