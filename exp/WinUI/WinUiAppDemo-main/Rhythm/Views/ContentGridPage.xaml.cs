using Microsoft.UI.Xaml.Controls;
using Rhythm.ViewModels;

namespace Rhythm.Views;

public sealed partial class ContentGridPage : Page
{
    public static readonly string PageName = "Content Grid";

    public static readonly bool IsPageHidden = false;

    public ContentGridViewModel ViewModel
    {
        get;
    }

    public ContentGridPage()
    {
        ViewModel = App.GetService<ContentGridViewModel>();
        InitializeComponent();
    }
}
