using Microsoft.AspNetCore.Components;

// Remember to replace the namespace below with your own project's namespace..
namespace FluentUI.Shared;


/// <summary />
public partial class Spacer
{
    /// <summary>
    /// Gets or sets the width of the spacer (in pixels)
    /// </summary>
    [Parameter]
    public int? Width { get; set; }
}
