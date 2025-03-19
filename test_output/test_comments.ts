// This is a comment
import { Component } from '@angular/core';

/**
 * Multi-line comment that should be removed
 * with multiple lines
 */
@Component({
  selector: 'app-root',
  template: `
    <div>
      <!-- HTML comment that should be preserved -->
      <h1>{{ title }}</h1>
      <p>Welcome to {{ title }}!</p>
    </div>
  `,
  styles: [`
    h1 {
      color: blue; /* CSS comment that should be preserved */
    }
  `]
})
export class AppComponent {
  title = 'TypeScript App'; // End of line comment

  /**
   * This is a JSDoc comment that should be preserved
   * @param value - The input value
   * @returns The processed value
   */
  processValue(value: string): string {
    // This comment should be removed
    const processed = value.trim(); // Another comment to remove

    /* This is another multi-line comment
     * that should be removed from the code
     */
    return processed;
  }

  // aicodeanalyzer: ignore
  ignoredFunction(): void {
    console.log('This function has an ignored comment');
  }

  stringWithCommentPatterns(): void {
    const str1 = "This string contains // a comment-like pattern";
    const str2 = 'Single quotes with // comment pattern';
    const str3 = `Template literal with // comment pattern
      and multiline content with more // patterns`;
    
    // This comment should be removed
    const regex = /\/\/.*/; // Comment after regex that looks like comment
    
    const jsx = <div>/* This looks like a comment but is JSX */</div>;
  }
}